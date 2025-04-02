use super::verification::Verified;
use crypto::ed25519::{private::PrivateKey, public::PublicKey, signature::Signature};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedAiRequest {
    pub query: AiRequest,
    signature: Signature,
}

impl SignedAiRequest {
    pub fn verify(self) -> Result<Verified<SignedAiRequest>> {
        let query = bincode::serialize(&self.query)?;
        self.query.pubkey.verify(&query, &self.signature)?;
        Ok(Verified::new(self))
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiRequest {
    /// Timestamp of the request in seconds since the Unix epoch.
    pub timestamp: u64,
    pub seed: i32,
    pub message: String,
    pub history: Vec<History>,
    pub pubkey: PublicKey,
}

impl AiRequest {
    pub fn new(message: String, history: Vec<History>, pubkey: PublicKey) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            message,
            history,
            pubkey,
            seed: rand::random(),
        }
    }

    pub fn sign(self, private_key: &PrivateKey) -> Result<SignedAiRequest> {
        let query = bincode::serialize(&self)?;
        let signature = private_key.sign(&query);
        Ok(SignedAiRequest {
            query: self,
            signature,
        })
    }

    pub fn as_history(&self) -> Vec<History> {
        let mut history = self.history.clone();
        history.push(History {
            content: self.message.clone(),
            role: Role::User,
        });
        history
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct History {
    pub content: String,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
    System,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::System => write!(f, "system"),
        }
    }
}
