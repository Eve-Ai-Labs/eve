use crate::error::OrchestratorError;
use ai::Question;
use crypto::ed25519::public::PublicKey;
use futures::channel::mpsc::{Receiver, Sender};
use multiaddr::Multiaddr;
use p2p::etp::{FromETP, ToETP};
use tokio::sync::oneshot;
use types::{
    ai::{query::QueryId, request::SignedAiRequest, verification::Verified},
    cluster::ClusterInfoWithNodes,
    p2p::EveMessage,
};

pub type ApiSender = tokio::sync::mpsc::Sender<OrchRequest>;
pub type ApiReceiver = tokio::sync::mpsc::Receiver<OrchRequest>;

pub type ToP2P = Sender<ToETP<EveMessage>>;
pub type FromP2P = Receiver<FromETP<EveMessage>>;

pub type AiSender = Sender<Question>;

pub enum OrchRequest {
    Ask {
        request: Verified<SignedAiRequest>,
        tx: oneshot::Sender<Result<QueryId, OrchestratorError>>,
    },
    AddNode {
        address: Option<Multiaddr>,
        public_key: PublicKey,
        tx: oneshot::Sender<Result<(), OrchestratorError>>,
    },
    RemoveNode {
        public_key: PublicKey,
        tx: oneshot::Sender<Result<(), OrchestratorError>>,
    },
    ClusterInfo {
        tx: oneshot::Sender<ClusterInfoWithNodes>,
    },
    Airdrop {
        address: PublicKey,
        amount: u64,
        tx: oneshot::Sender<Result<(), OrchestratorError>>,
    },
}
