use p2p::error::MultiaddrError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeError {
    #[error("invalid sender")]
    InvalidSender,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("Ai error: {0}")]
    AiError(#[from] ai::error::AiError),
    #[error("Failed to sign response")]
    FailedToSignResponse(#[from] eyre::Error),
    #[error("Failed to send message")]
    P2PError,
    #[error("Invalid multiaddr: {0}")]
    InvalidMultiaddr(#[from] MultiaddrError),
}
