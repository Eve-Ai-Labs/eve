use crypto::ed25519::{private::PrivateKey, public::PublicKey};
use eyre::Error;
use multiaddr::{multihash::Multihash, Multiaddr, PeerId};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Node {
    pub key: PublicKey,
    #[serde(serialize_with = "serialize_peer")]
    #[serde(deserialize_with = "deserialize_peer")]
    pub peer_id: PeerId,
    pub connected: bool,
    pub address: Option<Multiaddr>,
}

impl Node {
    pub fn new(key: PublicKey, peer_id: PeerId, address: Option<Multiaddr>) -> Self {
        Self {
            key,
            peer_id,
            connected: false,
            address,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    pub address: Option<Multiaddr>,
    #[serde(serialize_with = "serialize_peer")]
    #[serde(deserialize_with = "deserialize_peer")]
    pub peer_id: PeerId,
    pub is_connected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClusterInfoWithNodes {
    pub cluster_info: ClusterInfo,
    pub nodes: HashMap<PeerId, Node>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ClusterInfo {
    pub orch_address: Vec<Multiaddr>,
    pub orch_pubkey: PublicKey,
    pub webrtc_certhash: Option<String>,
    pub nodes_count: usize,
}

impl ClusterInfo {
    pub fn find_quic(&self) -> Option<&Multiaddr> {
        self.orch_address.iter().find(|addr| {
            addr.iter()
                .any(|proto| matches!(proto, multiaddr::Protocol::QuicV1))
        })
    }

    pub fn find_webrtc(&self) -> Result<Option<Multiaddr>, Error> {
        let addr = self
            .orch_address
            .iter()
            .find(|addr| {
                addr.iter()
                    .any(|proto| matches!(proto, multiaddr::Protocol::WebRTCDirect))
            })
            .cloned();
        let mut addr = if let Some(addr) = addr {
            addr
        } else {
            return Ok(None);
        };

        let has_hash = addr
            .iter()
            .any(|p| matches!(p, multiaddr::Protocol::Certhash(_)));
        if !has_hash {
            if let Some(hash) = &self.webrtc_certhash {
                let hash = Multihash::from_bytes(&hex::decode(hash)?)?;
                addr.push(multiaddr::Protocol::Certhash(hash));
            }
        }
        Ok(Some(addr))
    }
}

impl Default for ClusterInfo {
    fn default() -> Self {
        Self {
            orch_address: vec![],
            orch_pubkey: PrivateKey::generate().public_key(),
            webrtc_certhash: None,
            nodes_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricsInfo {
    /// requests
    pub requests: u64,
    /// current requests in process
    pub processing: i64,
    /// errors count
    pub errors: u64,
    /// timeouts
    pub timeouts: u64,
    /// avg latency
    pub latency: f64,
}

pub fn deserialize_peer<'de, D>(deserializer: D) -> Result<PeerId, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    PeerId::from_str(s).map_err(de::Error::custom)
}

pub fn serialize_peer<S>(peer: &PeerId, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&peer.to_base58())
}
