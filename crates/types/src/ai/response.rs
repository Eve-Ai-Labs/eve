use super::verification::Verified;
use crypto::ed25519::{public::PublicKey, signature::Signature};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiResponse {
    /// Timestamp of the response in seconds since the Unix epoch.
    pub timestamp: u64,
    pub response: String,
    pub pubkey: PublicKey,
    pub request_signature: Signature,
    pub cost: u64,
}

impl AiResponse {
    pub fn sign(
        &self,
        private_key: &crypto::ed25519::private::PrivateKey,
    ) -> Result<SignedAiResponse, eyre::Error> {
        let response = bincode::serialize(&self)?;
        let signature = private_key.sign(&response);
        Ok(SignedAiResponse {
            node_response: self.clone(),
            signature,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedAiResponse {
    pub node_response: AiResponse,
    signature: Signature,
}

impl SignedAiResponse {
    pub fn verify(self) -> Result<Verified<SignedAiResponse>, eyre::Error> {
        let response = bincode::serialize(&self.node_response)?;
        self.node_response
            .pubkey
            .verify(&response, &self.signature)?;
        Ok(Verified::new(self))
    }

    pub fn node_key(&self) -> PublicKey {
        self.node_response.pubkey
    }
}
