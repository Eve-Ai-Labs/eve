use crate::key::EvePublicKey;
use futures::channel::oneshot::Sender;
use libp2p::{gossipsub::PublishError, Multiaddr, PeerId};

pub mod net;
pub mod nodes;
pub mod proto;
pub mod requests;

#[derive(Debug)]
pub enum FromETP<Msg> {
    Receive(PeerId, Msg),
    Connect(PeerId),
    Disconnect(PeerId),
}

#[derive(Debug)]
pub enum ToETP<Msg> {
    Whitelisted(EvePublicKey, Vec<Multiaddr>),
    RemoveFromWhitelist(EvePublicKey),
    Send {
        to: PeerId,
        message: Msg,
        on_received: Option<Sender<DeliveryResult>>,
    },
    Listeners(Sender<Vec<Multiaddr>>),
    Dial(PeerId, Multiaddr),
    Shutdown,
}

#[derive(Debug)]
pub enum DeliveryResult {
    Success,
    NotConnected,
    Timeout,
    PublishError(PublishError),
    Bincode(bincode::Error),
}

impl DeliveryResult {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    pub fn is_not_connected(&self) -> bool {
        matches!(self, Self::NotConnected)
    }

    pub fn is_error(&self) -> bool {
        !self.is_success()
    }
}
