use super::signature::Signature;
use eyre::{eyre, Error};
use serde::Serialize;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct PublicKey(pub(super) ed25519_dalek::VerifyingKey);

impl PublicKey {
    pub fn to_hex(&self) -> String {
        hex::encode(self.0.to_bytes())
    }

    pub fn bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), Error> {
        Ok(self.0.verify_strict(message, &signature.0)?)
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl TryFrom<[u8; 32]> for PublicKey {
    type Error = Error;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        let key = ed25519_dalek::VerifyingKey::from_bytes(&bytes)
            .map_err(|_| eyre!("Invalid public key"))?;
        Ok(Self(key))
    }
}

impl TryFrom<&str> for PublicKey {
    type Error = Error;
    fn try_from(bytes: &str) -> Result<Self, Self::Error> {
        let mut buf = [0u8; 32];
        hex::decode_to_slice(bytes.trim_start_matches("0x"), &mut buf)
            .map_err(|_| Error::msg("Invalid public key"))?;
        Self::try_from(buf)
    }
}

impl FromStr for PublicKey {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl Serialize for PublicKey {
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

impl<'de> ::serde::Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let encoded_key = <String>::deserialize(deserializer)?;
            PublicKey::try_from(encoded_key.as_str())
                .map_err(<D::Error as ::serde::de::Error>::custom)
        } else {
            let bytes = <[u8; 32]>::deserialize(deserializer)?;
            PublicKey::try_from(bytes).map_err(<D::Error as ::serde::de::Error>::custom)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_hex() {
        let bytes = [0u8; 32];
        let key = PublicKey::try_from(bytes).unwrap();
        assert_eq!(
            key.to_hex(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_display() {
        let bytes = [0u8; 32];
        let key = PublicKey::try_from(bytes).unwrap();
        assert_eq!(
            format!("{}", key),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_try_from_bytes() {
        let bytes = [0u8; 32];
        let key = PublicKey::try_from(bytes).unwrap();
        assert_eq!(
            key.to_hex(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_try_from_str() {
        let hex_str = "0000000000000000000000000000000000000000000000000000000000000000";
        let key = PublicKey::try_from(hex_str).unwrap();
        assert_eq!(key.to_hex(), hex_str);
    }

    #[test]
    fn test_serialize_human_readable() {
        let bytes = [0u8; 32];
        let key = PublicKey::try_from(bytes).unwrap();
        let json = serde_json::to_string(&key).unwrap();
        assert_eq!(
            json,
            "\"0000000000000000000000000000000000000000000000000000000000000000\""
        );
    }

    #[test]
    fn test_serialize_non_human_readable() {
        let bytes = [0u8; 32];
        let key = PublicKey::try_from(bytes).unwrap();
        let bin = bincode::serialize(&key).unwrap();
        assert_eq!(bin, bytes);
    }

    #[test]
    fn test_deserialize_human_readable() {
        let json = "\"0000000000000000000000000000000000000000000000000000000000000000\"";
        let key: PublicKey = serde_json::from_str(json).unwrap();
        assert_eq!(
            key.to_hex(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_deserialize_non_human_readable() {
        let bytes = [0u8; 32];
        let bin = bincode::serialize(&bytes).unwrap();
        let key: PublicKey = bincode::deserialize(&bin).unwrap();
        assert_eq!(
            key.to_hex(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_invalid_hex_str() {
        let invalid_hex_str = "invalid_hex_string";
        assert!(PublicKey::try_from(invalid_hex_str).is_err());
    }
}
