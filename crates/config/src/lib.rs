use eyre::{eyre, Context, Error, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;

pub mod api;
pub mod base;
pub mod db;
pub mod llm;
pub mod logging;
pub mod node;
pub mod orch;
pub mod p2p;
pub mod rpc;
pub mod tasks;
pub mod url;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "tp")]
pub enum Config {
    Orch(Box<orch::OrchConfig>),
    Node(Box<node::NodeConfig>),
}

impl Config {
    pub fn llm(&self) -> &llm::OllamaConfig {
        match self {
            Config::Node(node) => &node.llm,
            Config::Orch(orch) => &orch.llm,
        }
    }

    pub fn base(&self) -> &base::BaseConfig {
        match self {
            Config::Node(node) => &node.base,
            Config::Orch(orch) => &orch.base,
        }
    }

    pub fn logger(&self) -> &logging::LoggerConfig {
        match self {
            Config::Node(node) => &node.logger,
            Config::Orch(orch) => &orch.logger,
        }
    }
}

impl Config {
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let file = File::create(path)
            .with_context(|| eyre!("Saving the configuration. Path: {path:?}"))?;
        serde_yaml::to_writer(file, self)
            .with_context(|| eyre!("Saving the configuration. Path: {path:?}"))
    }
}

pub fn load_config<P: AsRef<std::path::Path>>(path: P) -> Result<Config, Error> {
    let config_str = std::fs::read_to_string(path).context("Failed to read config file")?;
    let config: Config =
        serde_yaml::from_str(&config_str).context("Failed to parse config file")?;
    Ok(config)
}
