use eyre::Error;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

#[derive(Clone, PartialEq, Eq)]
pub struct Signature(pub(super) ed25519_dalek::Signature);

impl Signature {
    pub fn to_hex(&self) -> String {
        hex::encode(self.0.to_bytes())
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<[u8; 64]> for Signature {
    fn from(bytes: [u8; 64]) -> Self {
        Self(ed25519_dalek::Signature::from_bytes(&bytes))
    }
}

impl TryFrom<&str> for Signature {
    type Error = Error;
    fn try_from(bytes: &str) -> Result<Self, Self::Error> {
        let mut buf = [0u8; 64];
        hex::decode_to_slice(bytes, &mut buf).map_err(|_| Error::msg("Invalid public key"))?;
        Ok(Self::from(buf))
    }
}

#[derive(Serialize, Deserialize)]
struct SignatureBytes([u8; 32], [u8; 32]);

impl From<&Signature> for SignatureBytes {
    fn from(sig: &Signature) -> Self {
        let bytes = sig.0.to_bytes();
        let mut pub_key = [0u8; 32];
        let mut sig: [u8; 32] = [0u8; 32];
        pub_key.copy_from_slice(&bytes[0..32]);
        sig.copy_from_slice(&bytes[32..64]);
        Self(pub_key, sig)
    }
}

impl From<&SignatureBytes> for Signature {
    fn from(sig: &SignatureBytes) -> Self {
        let mut bytes = [0u8; 64];
        bytes[0..32].copy_from_slice(&sig.0);
        bytes[32..64].copy_from_slice(&sig.1);
        Self(ed25519_dalek::Signature::from_bytes(&bytes))
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_hex())
        } else {
            SignatureBytes::from(self).serialize(serializer)
        }
    }
}

impl<'de> ::serde::Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let encoded_key = <String>::deserialize(deserializer)?;
            Signature::try_from(encoded_key.as_str())
                .map_err(<D::Error as ::serde::de::Error>::custom)
        } else {
            let bytes = SignatureBytes::deserialize(deserializer)?;
            Ok(Signature::from(&bytes))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ed25519::private::PrivateKey;

    #[test]
    fn test_signature_serialize_deserialize() {
        let key = PrivateKey::generate();
        let message: &[u8] = b"test message";
        let sig: Signature = key.sign(message);

        let serialized = serde_json::to_string(&sig).unwrap();
        let deserialized: Signature = serde_json::from_str(&serialized).unwrap();

        assert_eq!(sig.to_hex(), deserialized.to_hex());

        let serialized_bincode = bincode::serialize(&sig).unwrap();
        let deserialized_bincode: Signature = bincode::deserialize(&serialized_bincode).unwrap();

        assert_eq!(sig.to_hex(), deserialized_bincode.to_hex());
    }
}
