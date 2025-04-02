use p2p::task::PeerId;
#[cfg(feature = "err_poem")]
use poem::{error::ResponseError, http::StatusCode};
use storage::StorageError;
use thiserror::Error;
use tokio::task::JoinError;
use types::ai::query::QueryId;

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("Query {0} is already in progress")]
    QueryIsAlreadyInProgress(QueryId),
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    #[error("P2P error")]
    P2PError,
    #[error("Verifier error")]
    VerifierError,
    #[error("Node {0} is not in whitelist")]
    NodeIsNotInWhitelist(PeerId),
    #[error("System role is not allowed")]
    SystemRoleIsNotAllowed,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid sender")]
    InvalidSender,
    #[error("Node {0} is already in whitelist")]
    NodeIsAlreadyInWhitelist(PeerId),
    #[error("Common error: {0}")]
    EyreError(#[from] eyre::Error),
    #[error("Task error: {0}")]
    TaskError(#[from] JoinError),
}

#[cfg(feature = "err_poem")]
impl ResponseError for OrchestratorError {
    fn status(&self) -> StatusCode {
        match self {
            OrchestratorError::NodeIsNotInWhitelist(_) => StatusCode::BAD_GATEWAY,
            OrchestratorError::QueryIsAlreadyInProgress(_) => StatusCode::TOO_EARLY,
            OrchestratorError::SystemRoleIsNotAllowed => StatusCode::NOT_FOUND,
            OrchestratorError::StorageError(_) | OrchestratorError::P2PError => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            OrchestratorError::InvalidSignature | OrchestratorError::InvalidSender => {
                StatusCode::BAD_REQUEST
            }
            OrchestratorError::NodeIsAlreadyInWhitelist(_) => StatusCode::BAD_REQUEST,
            OrchestratorError::EyreError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            OrchestratorError::TaskError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            OrchestratorError::VerifierError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Error)]
pub enum EvaluatorError {
    #[error("Common error: {0}")]
    EyreError(#[from] eyre::Error),
    #[error("Storage error: {0}")]
    AiError(#[from] ai::error::AiError),
    #[error("Invalid AI response: {0}")]
    InvalidAiResponse(#[from] serde_json::Error),
    #[error("Invalid relevance: {0}")]
    InvalidRelevance(eyre::Error),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Invalid JSON: {0}")]
    InvalidJson(&'static str),
}

#[cfg(feature = "err_poem")]
impl ResponseError for EvaluatorError {
    fn status(&self) -> StatusCode {
        match self {
            EvaluatorError::AiError(_)
            | EvaluatorError::EyreError(_)
            | EvaluatorError::InvalidAiResponse(_)
            | EvaluatorError::InvalidRelevance(_) => StatusCode::INTERNAL_SERVER_ERROR,
            EvaluatorError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            EvaluatorError::InvalidJson(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
