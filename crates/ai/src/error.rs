use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[cfg(feature = "ollama")]
    #[error(transparent)]
    Ollama(#[from] ollama_rs::error::OllamaError),
    #[error("Internal error")]
    InternalError,
    #[cfg(feature = "ollama")]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}
