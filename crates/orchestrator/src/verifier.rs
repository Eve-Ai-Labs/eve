use crate::error::EvaluatorError;
use ai::{error::AiError, Ai, Answer, Question, QuestionOptions};
use crypto::ed25519::{private::PrivateKey, public::PublicKey};
use eyre::{bail, Error};
use std::{mem, sync::Arc};
use tokio::{
    select,
    sync::{mpsc::Receiver, oneshot::Sender},
};
use tracing::{error, warn};
use types::{
    ai::{
        query::{NodeResult, Query},
        request::History,
        response::SignedAiResponse,
        verification::{SignedVerificationResult, VerificationResult},
    },
    percent::Percent,
};

const SYSTEM_PROMPT: &str = "You act as an evaluator of an AI's performance. You will be provided with a conversation history between a human and an AI. Your task is to analyze the AI's response, assess its quality, and provide a brief verdict in JSON format consisting of two fields:
'relevance' — a number from 0 to 100, where 0 means the response is completely irrelevant, and 100 means the response fully meets expectations and is accurate.
'description' — a short textual explanation of the given score.
Return only a JSON object. Do not include any additional text or commentary before or after the JSON object.";

pub struct VerifierTask<A> {
    key: PrivateKey,
    ai: Arc<A>,
    receiver: Receiver<VerificationRequest>,
}

impl<A: Ai + Send + Sync + 'static> VerifierTask<A> {
    pub fn new(key: PrivateKey, ai: Arc<A>, receiver: Receiver<VerificationRequest>) -> Self {
        Self { key, ai, receiver }
    }

    async fn handle_request(&self, mut request: VerificationRequest) -> Result<(), Error> {
        let ai = self.ai.clone();
        let key = self.key.clone();

        let question = Question {
            message: mem::take(&mut request.question),
            history: vec![History {
                content: SYSTEM_PROMPT.to_owned(),
                role: types::ai::request::Role::System,
            }],
            options: QuestionOptions {
                seed: request.seed,
                ..Default::default()
            },
        };

        tokio::spawn(async move {
            let ai_response = ai.ask(question).await;
            match parse_ai_response(ai_response, key.clone(), request.node_response) {
                Ok(result) => request.on_result.send(result),
                Err((err, material)) => match map_error_to_response(err, key, material) {
                    Ok(resp) => request.on_result.send(resp),
                    Err(err) => {
                        warn!("Failed to map error to response: {}", err);
                        Ok(())
                    }
                },
            }
        });

        Ok(())
    }

    pub async fn run(&mut self) {
        loop {
            select! {
                Some(request) = self.receiver.recv() => {
                    if let Err(err) = self.handle_request(request).await {
                        error!("Error handling request: {}", err);
                    }
                }
            }
        }
    }
}

fn parse_ai_answer(
    answer: Result<Answer, AiError>,
) -> Result<(AiResponseJson, Percent), EvaluatorError> {
    let answer = match answer {
        Ok(answer) => answer.message,
        Err(err) => {
            return Err(EvaluatorError::AiError(err));
        }
    };

    let start_json = answer
        .find("{")
        .ok_or_else(|| EvaluatorError::InvalidJson("No opening bracket found"))?;
    let end_json = answer
        .rfind("}")
        .ok_or_else(|| EvaluatorError::InvalidJson("No closing bracket found"))?;
    let answer = &answer[start_json..=end_json];

    let answer = serde_json::from_str::<AiResponseJson>(answer.trim())?;
    let relevance =
        Percent::try_from(answer.relevance).map_err(EvaluatorError::InvalidRelevance)?;
    Ok((answer, relevance))
}

fn parse_ai_response(
    answer: Result<Answer, AiError>,
    key: PrivateKey,
    material: SignedAiResponse,
) -> Result<VerificationResponse, (EvaluatorError, SignedAiResponse)> {
    let verification_result = match parse_ai_answer(answer) {
        Ok((answer, relevance)) => VerificationResult {
            material,
            inspector: key.public_key(),
            relevance,
            description: answer.description,
        },
        Err(err) => {
            return Err((err, material));
        }
    };

    Ok(VerificationResponse {
        node_key: verification_result.material.node_key(),
        verification_result: verification_result
            .sign(&key)
            .map_err(|(res, err)| (err.into(), res.material))?,
    })
}

fn map_error_to_response(
    err: EvaluatorError,
    key: PrivateKey,
    material: SignedAiResponse,
) -> Result<VerificationResponse, Error> {
    let result = VerificationResult {
        material,
        inspector: key.public_key(),
        relevance: Percent::zero(),
        description: format!("Failed to evaluate AI response: {}", err),
    };

    Ok(VerificationResponse {
        node_key: result.material.node_key(),
        verification_result: result.sign(&key).map_err(|(_, err)| err)?,
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AiResponseJson {
    relevance: u8,
    description: String,
}

pub struct VerificationRequest {
    pub seed: i32,
    pub question: String,
    pub on_result: Sender<VerificationResponse>,
    pub node_response: SignedAiResponse,
}

impl VerificationRequest {
    pub fn new(
        query: &Query,
        node_key: PublicKey,
        on_result: Sender<VerificationResponse>,
    ) -> Result<Self, Error> {
        let node_response = query
            .response
            .iter()
            .find(|response| response.node_key() == node_key)
            .ok_or_else(|| eyre::eyre!("Node response not found"))?
            .clone();
        let node_response = if let NodeResult::NodeResponse(node_result) = node_response {
            node_result
        } else {
            bail!("Node response not found");
        };

        Ok(Self {
            question: Self::prepare_question(query, node_key)?,
            on_result,
            seed: query.request.query.seed,
            node_response,
        })
    }

    fn prepare_question(query: &Query, node_key: PublicKey) -> Result<String, Error> {
        let mut request = String::new();

        let id = query.id.to_hex();
        request.push_str(&format!("id: {}\n", id));
        request.push_str(&format!("history section start {}\n", id));
        query.request.query.history.iter().for_each(|message| {
            request.push_str(&format!("{}:\n{}\n", message.role, message.content));
        });
        request.push_str(&format!("history section end {}\n", id));

        request.push_str(&format!(
            "user request with id {}:\n{}\n",
            id, query.request.query.message
        ));

        let node_response = query
            .response
            .iter()
            .find(|response| response.node_key() == node_key);

        if let Some(NodeResult::NodeResponse(resp)) = node_response {
            request.push_str(&format!(
                "ai response with id {}:\n{}\n",
                id, resp.node_response.response
            ));
        } else {
            bail!("Node response not found");
        }

        Ok(request)
    }
}

pub struct VerificationResponse {
    pub node_key: PublicKey,
    pub verification_result: SignedVerificationResult,
}
