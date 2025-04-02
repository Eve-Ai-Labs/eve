use super::response::SignedAiResponse;
use crate::percent::Percent;
use crypto::ed25519::{private::PrivateKey, public::PublicKey, signature::Signature};
use eyre::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Verified<T>(T);

impl<T> Verified<T> {
    pub(super) fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> AsRef<T> for Verified<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedVerificationResult {
    pub result: VerificationResult,
    signature: Signature,
}

impl SignedVerificationResult {
    pub fn verify(&self) -> Result<()> {
        let buf: Vec<u8> = bincode::serialize(&self.result)?;
        self.result.inspector.verify(&buf, &self.signature)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationResult {
    pub material: SignedAiResponse,
    pub inspector: PublicKey,
    pub relevance: Percent,
    pub description: String,
}

impl VerificationResult {
    pub fn sign(
        self,
        private_key: &PrivateKey,
    ) -> Result<SignedVerificationResult, (Self, eyre::Error)> {
        let buf = match bincode::serialize(&self) {
            Ok(buf) => buf,
            Err(err) => {
                return Err((
                    self,
                    eyre::eyre!("Failed to serialize verification result: {}", err),
                ));
            }
        };

        let signature = private_key.sign(&buf);
        if self.inspector != private_key.public_key() {
            return Err((
                self,
                eyre::eyre!("Inspector public key does not match private key"),
            ));
        }

        Ok(SignedVerificationResult {
            result: self,
            signature,
        })
    }
}
