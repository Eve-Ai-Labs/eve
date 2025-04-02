use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RpcConfig {
    pub address: SocketAddr,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            address: default_rpc_address(),
        }
    }
}

pub fn default_rpc_address() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 1733))
}
