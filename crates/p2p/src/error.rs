use crate::etp::nodes::State;
pub use libp2p::multiaddr::Error as MultiaddrError;
use libp2p::{swarm::DialError, Multiaddr, PeerId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EtpError {
    #[error("Already connected to peer: {0:?}")]
    AlreadyConnected(PeerId),
    #[error("Invalid state:{0:?} {1:?}")]
    InvalidState(PeerId, State),
    #[error("Peer is not in the whitelist: {0:?}")]
    NotWhitelisted(PeerId),
    #[error("Failed to send message to app")]
    AppError,
    #[error("Invalid action")]
    InvalidAction,
    #[error("Common error")]
    Common(#[from] eyre::Error),
    #[error("Failed to dial node:{0:?} {1:?}")]
    DialError(DialError, Multiaddr),
    #[error("Failed to disconnect peer:{0:?}")]
    DisconnectError(PeerId),
    #[error("Unknown sender")]
    UnknownSender,
    #[error("Unknown topic")]
    UnknownTopic,
}

impl EtpError {
    pub fn is_app_error(&self) -> bool {
        matches!(self, EtpError::AppError)
    }

    pub fn is_invalid_state(&self) -> Option<PeerId> {
        match self {
            EtpError::InvalidState(peer_id, _) => Some(*peer_id),
            _ => None,
        }
    }
}
