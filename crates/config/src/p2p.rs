use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeP2PConfig {
    pub address: Vec<Multiaddr>,
    pub orch_address: Multiaddr,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrchP2PConfig {
    pub address: Vec<Multiaddr>,
}
