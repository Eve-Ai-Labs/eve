use crate::ai::{query::QueryId, request::SignedAiRequest, response::SignedAiResponse};
use crypto::ed25519::public::PublicKey;
use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum EveMessage {
    Orch(OrchMessage),
    Node(NodeMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrchMessage {
    AiRequest {
        id: QueryId,
        request: SignedAiRequest,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NodeMessage {
    AiResponse {
        id: QueryId,
        response: Result<SignedAiResponse, String>,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Peer {
    pub address: Option<Multiaddr>,
    pub public_key: PublicKey,
}
