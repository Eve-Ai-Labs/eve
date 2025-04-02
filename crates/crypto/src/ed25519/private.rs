use super::{public::PublicKey, signature::Signature};
use ed25519_dalek::ed25519::signature::Signer;
use eyre::Error;
use serde::Serialize;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

#[derive(Clone, Eq)]
pub struct PrivateKey(ed25519_dalek::SigningKey);

impl PartialEq for PrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_bytes() == other.0.as_bytes()
    }
}

impl Display for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*** private key ***")
    }
}

impl PrivateKey {
    pub fn to_hex(&self) -> String {
        hex::encode(self.0.to_bytes())
    }

    pub fn bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.verifying_key())
    }

    pub fn generate() -> Self {
        Self(ed25519_dalek::SigningKey::generate(&mut rand::thread_rng()))
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        Signature(self.0.sign(message))
    }
}

impl Debug for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PrivateKey(,,¬﹏¬,,)")
    }
}

impl TryFrom<[u8; 32]> for PrivateKey {
    type Error = Error;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        Ok(Self(ed25519_dalek::SigningKey::from_bytes(&bytes)))
    }
}

impl TryFrom<&str> for PrivateKey {
    type Error = Error;
    fn try_from(bytes: &str) -> Result<Self, Self::Error> {
        let mut buf = [0u8; 32];
        hex::decode_to_slice(bytes.trim_start_matches("0x"), &mut buf)
            .map_err(|_| Error::msg("Invalid secret key"))?;
        Self::try_from(buf)
    }
}

impl FromStr for PrivateKey {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl Serialize for PrivateKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_hex())
        } else {
            self.0.to_bytes().serialize(serializer)
        }
    }
}

impl<'de> ::serde::Deserialize<'de> for PrivateKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let encoded_key = <String>::deserialize(deserializer)?;
            PrivateKey::try_from(encoded_key.as_str())
                .map_err(<D::Error as ::serde::de::Error>::custom)
        } else {
            let bytes = <[u8; 32]>::deserialize(deserializer)?;
            PrivateKey::try_from(bytes).map_err(<D::Error as ::serde::de::Error>::custom)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_hex() {
        let secret_key_bytes = [0u8; 32];
        let secret_key = PrivateKey::try_from(secret_key_bytes).unwrap();
        assert_eq!(
            secret_key.to_hex(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_bytes() {
        let secret_key_bytes = [0u8; 32];
        let secret_key = PrivateKey::try_from(secret_key_bytes).unwrap();
        assert_eq!(secret_key.bytes(), secret_key_bytes);
    }

    #[test]
    fn test_public_key() {
        let secret_key_bytes = [0u8; 32];
        let secret_key = PrivateKey::try_from(secret_key_bytes).unwrap();
        let public_key = secret_key.public_key();
        assert_eq!(public_key.0.to_bytes().len(), 32);
    }

    #[test]
    fn test_display() {
        let secret_key_bytes = [0u8; 32];
        let secret_key = PrivateKey::try_from(secret_key_bytes).unwrap();
        assert_eq!(
            secret_key.to_hex(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_try_from_bytes() {
        let secret_key_bytes = [0u8; 32];
        let secret_key = PrivateKey::try_from(secret_key_bytes).unwrap();
        assert_eq!(secret_key.bytes(), secret_key_bytes);
    }

    #[test]
    fn test_try_from_str() {
        let secret_key_str = "0000000000000000000000000000000000000000000000000000000000000000";
        let secret_key = PrivateKey::try_from(secret_key_str).unwrap();
        assert_eq!(secret_key.to_hex(), secret_key_str);
    }

    #[test]
    fn test_serialize_human_readable() {
        let secret_key_bytes = [0u8; 32];
        let secret_key = PrivateKey::try_from(secret_key_bytes).unwrap();
        let serialized = serde_json::to_string(&secret_key).unwrap();
        assert_eq!(
            serialized,
            "\"0000000000000000000000000000000000000000000000000000000000000000\""
        );
    }

    #[test]
    fn test_deserialize_human_readable() {
        let secret_key_str = "\"0000000000000000000000000000000000000000000000000000000000000000\"";
        let secret_key: PrivateKey = serde_json::from_str(secret_key_str).unwrap();
        assert_eq!(
            secret_key.to_hex(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_serialize_non_human_readable() {
        let secret_key_bytes = [0u8; 32];
        let secret_key = PrivateKey::try_from(secret_key_bytes).unwrap();
        let serialized = bincode::serialize(&secret_key).unwrap();
        assert_eq!(serialized, secret_key_bytes);
    }

    #[test]
    fn test_deserialize_non_human_readable() {
        let secret_key_bytes = [0u8; 32];
        let serialized = bincode::serialize(&secret_key_bytes).unwrap();
        let secret_key: PrivateKey = bincode::deserialize(&serialized).unwrap();
        assert_eq!(secret_key.bytes(), secret_key_bytes);
    }
}
