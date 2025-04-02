use jwt::JwtSecret;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub const JWT_DEFAULT_DEV: &str =
    "c9ec179a3fbc9f22cb2370fef360604235f412ac953d9bb2f5616deb7d98bc74";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub blacklist_words: Vec<String>,
    pub req_per_hour: u64,
    pub airdrop_per_hour: u64,
    pub max_req_length: usize,
    pub jwt: JwtSecret,
    pub cluster_info_ttl_secs: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            blacklist_words: Vec::new(),
            req_per_hour: 100,
            max_req_length: 10000,
            jwt: JwtSecret::from_str(JWT_DEFAULT_DEV).unwrap(),
            cluster_info_ttl_secs: 10,
            airdrop_per_hour: 10,
        }
    }
}
