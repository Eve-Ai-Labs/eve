use crate::{base, llm, logging, p2p};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    #[serde(default)]
    pub base: base::BaseConfig,
    #[serde(default)]
    pub llm: llm::OllamaConfig,
    #[serde(default)]
    pub logger: logging::LoggerConfig,
    pub p2p: p2p::NodeP2PConfig,
}
