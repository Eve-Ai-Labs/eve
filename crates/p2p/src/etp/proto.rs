use crypto::ed25519::public::PublicKey;
use libp2p::PeerId;
use rand::random;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Ack;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct MessageId(u64);

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageId {
    pub fn new() -> Self {
        Self(random())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolMessage<Msg> {
    /// The public key of the destination.
    pub to: PublicKey,
    /// Message identifier.
    pub id: MessageId,
    /// The message to send.
    pub etm: ETM<Msg>,
}

impl<Msg> ProtocolMessage<Msg> {
    pub fn new(to: PublicKey, etm: ETM<Msg>) -> Self {
        Self {
            to,
            id: MessageId::new(),
            etm,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ETM<Msg> {
    /// Send a message to a peer.
    Send(Msg),
    /// Notify a peer about disconnected peer.
    Disconnected,
    /// Notify a peer about ready peer.
    Connected(Caller),
    /// Notify a peer about reconnected peer.
    ReConnect,
    /// Ping a peer.
    Ping,
    /// Ack a message.
    Ack(MessageId),
}

impl<Msg> ETM<Msg> {
    pub fn as_ack(&self) -> Option<&MessageId> {
        match self {
            ETM::Ack(id) => Some(id),
            _ => None,
        }
    }

    pub fn tp(&self) -> EtmType {
        match self {
            ETM::Send(_) => EtmType::Send,
            ETM::Disconnected => EtmType::Disconnected,
            ETM::Connected(_) => EtmType::Connected,
            ETM::ReConnect => EtmType::ReConnect,
            ETM::Ping => EtmType::Ping,
            ETM::Ack(_) => EtmType::Ack,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Caller(PeerId);

impl From<PeerId> for Caller {
    fn from(peer_id: PeerId) -> Self {
        Self(peer_id)
    }
}

impl From<Caller> for PeerId {
    fn from(caller: Caller) -> Self {
        caller.0
    }
}

impl AsRef<PeerId> for Caller {
    fn as_ref(&self) -> &PeerId {
        &self.0
    }
}

#[derive(Debug)]
pub enum EtmType {
    Send,
    Disconnected,
    Connected,
    ReConnect,
    Ping,
    Ack,
}
