#![cfg(feature = "ollama")]
use ai::{Ai, Question, QuestionOptions};
use node_config::llm::OllamaConfig;
use std::time::Duration;
use types::ai::request::{History, Role};

// #[tokio::test]
pub async fn test_throttle() {
    tracing_subscriber::fmt::init();

    let config = OllamaConfig {
        model: "qwen2.5-coder:0.5b".to_string(),
        ..Default::default()
    };
    let llm = ai::ollama::Llm::new(&config).await.unwrap();

    let mut history = Vec::new();

    let time = std::time::Instant::now();

    let count = 10;

    for i in 0..count {
        let message = "do nothing, reply empty".to_string();
        history.push(History {
            content: message.clone(),
            role: Role::User,
        });
        let q = Question {
            message,
            history: history.clone(),
            options: QuestionOptions::default(),
        };
        tracing::info!(i, "Sending request...");
        let response = llm.ask(q).await.unwrap();
        tracing::info!(i, "Response: {}", response.message);
        history.push(History {
            content: response.message,
            role: Role::Assistant,
        });
    }

    let elapsed = time.elapsed();
    tracing::info!("Elapsed time: {:?}", elapsed);
    assert!(elapsed >= Duration::from_secs(count / 2));
}

// #[tokio::test]
pub async fn test_retry() {
    tracing_subscriber::fmt::init();

    let retry_limit = 5;

    let config = OllamaConfig {
        retry_limit,
        model: "qwen2.5-coder:0.5b".to_string(),
        ..Default::default()
    };

    let mut llm = ai::ollama::Llm::new(&config).await.unwrap();
    tracing::info!("LLM initialized");

    llm.update_model("unexisting-model:0".to_string());

    let res = llm
        .ask(Question {
            message: "do nothing, reply empty".to_string(),
            history: Vec::new(),
            options: QuestionOptions::default(),
        })
        .await;
    assert!(res.is_err());
}
