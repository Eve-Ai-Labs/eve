use super::{
    request::{History, Role, SignedAiRequest},
    response::SignedAiResponse,
    verification::SignedVerificationResult,
};
use crate::percent::Percent;
use crypto::{
    ed25519::public::PublicKey,
    hash::{sha3, Hash},
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Query {
    pub id: QueryId,
    pub sequence: u64,
    pub request: SignedAiRequest,
    pub response: Vec<NodeResult>,
}

pub fn query_id(sequence: u64, request: &SignedAiRequest) -> QueryId {
    sha3(&(sequence, &request))
}

impl Query {
    pub fn new(id: QueryId, sequence: u64, request: SignedAiRequest) -> Self {
        Self {
            id,
            sequence,
            request,
            response: Vec::new(),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.response.iter().all(|result| match result {
            NodeResult::SentRequest(_) | NodeResult::NodeResponse(_) => false,
            NodeResult::Verified(_) | NodeResult::Timeout(_) | NodeResult::Error(_, _) => true,
        })
    }

    pub fn as_history(&self) -> Vec<History> {
        let mut history = self.request.query.as_history();
        let node_response = self
            .response
            .iter()
            .filter_map(|v| match v {
                NodeResult::Verified(v) => Some((
                    v.result.relevance.clone(),
                    v.result.material.node_response.response.clone(),
                )),
                _ => None,
            })
            .fold((Percent::zero(), None), |b, value| {
                if b.1.is_none() || b.0 < value.0 {
                    (value.0, Some(value.1))
                } else {
                    b
                }
            })
            .1;
        if let Some(node_response) = node_response {
            history.push(History {
                content: node_response,
                role: Role::Assistant,
            });
        }

        history
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeResult {
    SentRequest(PublicKey),
    Timeout(Box<NodeResult>),
    NodeResponse(SignedAiResponse),
    Error(PublicKey, String),
    Verified(Box<SignedVerificationResult>),
}

impl NodeResult {
    pub fn is_sent_request(&self) -> bool {
        matches!(self, NodeResult::SentRequest(_))
    }

    pub fn is_node_response(&self) -> bool {
        matches!(self, NodeResult::NodeResponse(_))
    }

    pub fn is_verified(&self) -> bool {
        matches!(self, NodeResult::Verified(_))
    }

    pub fn verified(&self) -> Option<&SignedVerificationResult> {
        match self {
            NodeResult::Verified(result) => Some(result),
            _ => None,
        }
    }

    pub fn node_key(&self) -> PublicKey {
        match self {
            NodeResult::SentRequest(key) => *key,
            NodeResult::NodeResponse(response) => response.node_response.pubkey,
            NodeResult::Verified(approval) => approval.result.material.node_response.pubkey,
            NodeResult::Timeout(inner) => inner.node_key(),
            NodeResult::Error(public_key, _) => *public_key,
        }
    }

    pub fn is_timeout(&self) -> bool {
        matches!(self, NodeResult::Timeout(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, NodeResult::Error(_, _))
    }

    pub fn as_node_response(&self) -> Option<&SignedAiResponse> {
        match self {
            NodeResult::NodeResponse(response) => Some(response),
            _ => None,
        }
    }

    pub fn as_verified_response(&self) -> Option<&SignedVerificationResult> {
        match self {
            NodeResult::Verified(response) => Some(response),
            _ => None,
        }
    }
}

impl PartialOrd for NodeResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self == other {
            return Ordering::Equal;
        }
        match self {
            NodeResult::Error(..) => Ordering::Greater,
            NodeResult::SentRequest(..) => match other {
                NodeResult::Error(..) => Ordering::Less,
                _ => Ordering::Greater,
            },
            NodeResult::Timeout(..) => match other {
                NodeResult::Error(..) | NodeResult::SentRequest(..) => Ordering::Less,
                _ => Ordering::Greater,
            },
            NodeResult::NodeResponse(..) => match other {
                NodeResult::Error(..) | NodeResult::SentRequest(..) | NodeResult::Timeout(..) => {
                    Ordering::Less
                }
                _ => Ordering::Greater,
            },
            NodeResult::Verified(a) => match other {
                NodeResult::Verified(b) => a.result.relevance.cmp(&b.result.relevance),
                _ => Ordering::Less,
            },
        }
    }
}

pub type QueryId = Hash;
