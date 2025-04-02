use serde::Serialize;
use sha3::{Digest, Sha3_256};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Hash([u8; 32]);

pub fn sha3<V: Serialize>(value: &V) -> Hash {
    let mut hasher = Sha3_256::new();
    bincode::serialize_into(&mut hasher, value).unwrap();
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    Hash(hash)
}

pub fn sha3_bytes(bytes: &[u8]) -> Hash {
    let mut hasher = Sha3_256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    Hash(hash)
}

pub fn sha3_with_seed<V: Serialize>(value: &V, seed: &[u8]) -> Hash {
    let mut hasher = Sha3_256::new();
    hasher.update(seed);
    bincode::serialize_into(&mut hasher, value).unwrap();
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    Hash(hash)
}

impl Hash {
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }
}

impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl TryFrom<&str> for Hash {
    type Error = eyre::Error;
    fn try_from(bytes: &str) -> Result<Self, Self::Error> {
        let mut buf = [0u8; 32];
        hex::decode_to_slice(bytes.trim_start_matches("0x"), &mut buf)
            .map_err(|_| eyre::eyre!("Invalid hash"))?;
        Ok(Self(buf))
    }
}

impl FromStr for Hash {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Hash::try_from(s)
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> serde::Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let encoded_key = <String>::deserialize(deserializer)?;
        Hash::try_from(encoded_key.as_str()).map_err(<D::Error as serde::de::Error>::custom)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_to_hex() {
        let bytes = [0u8; 32];
        let hash = Hash::from(bytes);
        assert_eq!(
            hash.to_hex(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_hash_to_bytes() {
        let bytes = [1u8; 32];
        let hash = Hash::from(bytes);
        assert_eq!(hash.to_bytes(), bytes);
    }

    #[test]
    fn test_sha3() {
        let value = "test";
        let hash = sha3(&value);
        assert_eq!(
            hash.to_hex(),
            "c3e53ee0e3b2655fb8658831847e5685a30e2c5a5ea83f675b97abe7cb1fc599"
        );
    }

    #[test]
    fn test_sha3_bytes() {
        let bytes = b"test";
        let hash = sha3_bytes(bytes);
        assert_eq!(
            hash.to_hex(),
            "36f028580bb02cc8272a9a020f4200e346e276ae664e45ee80745574e2f5ab80"
        );
    }

    #[test]
    fn test_try_from_str() {
        let hex_str = "36e9f3d4c7c8e3b1d8e8f3d4c7c8e3b1d8e8f3d4c7c8e3b1d8e8f3d4c7c8e3b1";
        let hash = Hash::try_from(hex_str).unwrap();
        assert_eq!(hash.to_hex(), hex_str);
    }

    #[test]
    fn test_serialize() {
        let bytes = [1u8; 32];
        let hash = Hash::from(bytes);
        let serialized = serde_json::to_string(&hash).unwrap();
        assert_eq!(
            serialized,
            "\"0101010101010101010101010101010101010101010101010101010101010101\""
        );
    }

    #[test]
    fn test_deserialize() {
        let hex_str = "0101010101010101010101010101010101010101010101010101010101010101";
        let json_str = format!("\"{}\"", hex_str);
        let hash: Hash = serde_json::from_str(&json_str).unwrap();
        assert_eq!(hash.to_hex(), hex_str);
    }
}
