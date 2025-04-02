use crate::{api::ApiConfig, base, db, llm, logging, p2p, rpc, tasks::AiTasksConfig};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrchConfig {
    #[serde(default)]
    pub base: base::BaseConfig,
    #[serde(default)]
    pub llm: llm::OllamaConfig,
    #[serde(default)]
    pub logger: logging::LoggerConfig,
    #[serde(default)]
    pub db: db::DbConfig,
    #[serde(default)]
    pub rpc: rpc::RpcConfig,
    #[serde(default)]
    pub ai_tasks: AiTasksConfig,
    #[serde(default)]
    pub api: ApiConfig,
    pub p2p: p2p::OrchP2PConfig,
}
