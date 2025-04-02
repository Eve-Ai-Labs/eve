extern crate ollama_rs;

use crate::{error::AiError, Ai, Answer, Question};
use backon::{FibonacciBuilder, Retryable};
use node_config::llm::OllamaConfig;
use ollama_rs::{
    error::OllamaError,
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage},
        options::GenerationOptions,
    },
    Ollama,
};
use ratelimit::Ratelimiter;
use std::{iter::once, sync::Arc, time::Duration};
use tracing::warn;
use types::ai::request::{History, Role};

#[derive(Clone)]
pub struct Llm {
    ollama: Ollama,
    model: String,
    limiter: Arc<Ratelimiter>,
    retry_limit: usize,
}

impl Llm {
    pub async fn new(config: &OllamaConfig) -> Result<Self, AiError> {
        let port = config.url.port_or_known_default().unwrap_or(11434);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()?;
        let ollama = Ollama::new_with_client(config.url.clone(), port, client);

        if config.pull_model {
            (|| async {
                let models = ollama.list_local_models().await?;
                if !models.iter().any(|m| m.name == config.model) {
                    tracing::info!("pulling `{}` model", config.model);
                    ollama.pull_model(config.model.clone(), false).await?;
                }
                Ok(()) as Result<(), OllamaError>
            })
            .retry(FibonacciBuilder::default().with_max_times(config.retry_limit))
            .notify(|e, _| {
                tracing::error!(%e,"error when request ollama");
            })
            .when(|e| match e {
                OllamaError::ReqwestError(err) => err.is_timeout() || err.is_connect(),
                _ => false,
            })
            .await?;
        }

        let limiter = Arc::new(
            Ratelimiter::builder(
                config.req_per_time,
                Duration::from_millis(config.time_millis),
            )
            .max_tokens(config.max_tokens)
            .initial_available(config.req_per_time)
            .build()
            .unwrap(),
        );
        Ok(Llm {
            ollama,
            model: config.model.clone(),
            limiter,
            retry_limit: config.retry_limit,
        })
    }

    pub fn update_model(&mut self, model: String) {
        self.model = model;
    }
}

impl Ai for Llm {
    async fn ask(&self, ask: Question) -> Result<Answer, AiError> {
        let Question {
            message,
            history,
            options,
        } = ask;
        let msg_len = message.len();
        let history = history
            .into_iter()
            .map(|History { content, role }| match role {
                Role::User => ChatMessage::user(content),
                Role::Assistant => ChatMessage::assistant(content),
                Role::System => ChatMessage::system(content),
            })
            .chain(once(ChatMessage::user(message)))
            .collect::<Vec<ChatMessage>>();
        let req = ChatMessageRequest::new(self.model.clone(), history).options(
            GenerationOptions::default()
                .temperature(options.temperature)
                .seed(options.seed),
        );

        if let Err(sleep) = self.limiter.try_wait() {
            tracing::warn!(?sleep, "Rate limit exceeded");
            tokio::time::sleep(sleep).await;
        }

        let response = (|| async { self.ollama.send_chat_messages(req.clone()).await })
            .retry(FibonacciBuilder::default().with_max_times(self.retry_limit))
            .notify(|e, _| {
                tracing::error!(%e,"error when request ollama");
            })
            .when(|e| {
                if let OllamaError::ReqwestError(err) = e {
                    err.is_timeout()
                } else {
                    true
                }
            })
            .await?;

        let tokens = match response.final_data {
            Some(data) => (data.prompt_eval_count + data.eval_count) as u64,
            None => {
                warn!("response.final_data is None!");
                (response.message.content.len() + msg_len) as u64
            }
        };

        Ok(Answer {
            message: response.message.content,
            tokens,
        })
    }
}
