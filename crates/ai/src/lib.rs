pub mod error;
#[cfg(feature = "ollama")]
pub mod ollama;

use error::AiError;
use types::ai::request::History;

#[derive(Debug)]
pub struct QuestionOptions {
    pub seed: i32,
    pub temperature: f32,
}

impl Default for QuestionOptions {
    fn default() -> Self {
        Self {
            seed: 0,
            temperature: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct Question {
    pub message: String,
    pub history: Vec<History>,
    pub options: QuestionOptions,
}

impl Question {
    pub fn length(&self) -> usize {
        self.history.len()
            + self.history.iter().map(|h| h.content.len()).sum::<usize>()
            + self.message.len()
    }
}

#[derive(Debug)]
pub struct Answer {
    pub message: String,
    pub tokens: u64,
}

pub trait Ai {
    fn ask(
        &self,
        ask: Question,
    ) -> impl std::future::Future<Output = Result<Answer, AiError>> + Send;
}
