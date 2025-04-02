use serde::{Deserialize, Serialize};
use types::p2p::Peer;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiTasksConfig {
    pub whitelist: Vec<Peer>,
    pub replication_factor: u64,
    pub task_timeout_secs: u64,
}

impl Default for AiTasksConfig {
    fn default() -> Self {
        Self {
            whitelist: vec![],
            replication_factor: 3,
            task_timeout_secs: 60,
        }
    }
}
