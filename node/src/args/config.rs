use crate::init_logger;
use clap::Parser;
use color_eyre::eyre::{eyre, Context, Error, Result};
use crypto::ed25519::private::PrivateKey;
use multiaddr::Multiaddr;
use node_config::{
    base::BaseConfig, llm::OllamaConfig, node::NodeConfig, p2p::NodeP2PConfig, Config,
};
use std::{
    fs::{self, create_dir_all},
    path::PathBuf,
};
use types::cluster::ClusterInfo;
use url::Url;

const NODE_PATH_DEFAULT: &str = "./eve";

/// Initialize the configuration for the node
#[derive(Debug, Parser)]
pub(crate) struct CfgNode {
    /// The directory for the node. Default is ./eve
    #[arg(default_value = NODE_PATH_DEFAULT)]
    path: PathBuf,

    /// URL to the ollama. Default is http://localhost:11434
    #[arg(short = 'l', long, default_value = "http://localhost:11434")]
    ollama_url: Url,

    /// Model to use for the ollama. Default is deepseek-r1:1.5b
    #[arg(short, long, default_value = "deepseek-r1:1.5b")]
    ai_model: String,

    /// Orchestrator RPC address
    /// Example: http://127.0.0.1:1133
    #[arg(short, long)]
    orch: Url,

    /// p2p multiaddress
    /// Example: /ip4/0.0.0.0/udp/0/quic-v1 or /ip4/0.0.0.0/udp/0/webrtc-direct
    #[arg(short, long)]
    p2p_address: Vec<Multiaddr>,
}

impl CfgNode {
    pub(crate) async fn execute(self) -> Result<()> {
        init_logger(None);

        let info = self.load_cluster_info().await?;

        let self_key = PrivateKey::generate();

        let llm = OllamaConfig {
            url: self.ollama_url.clone(),
            model: self.ai_model.clone(),
            ..Default::default()
        };

        let cfg = NodeConfig {
            base: BaseConfig {
                key: self_key.clone(),
                pub_key: self_key.public_key(),
                orch_pub_key: info.orch_pubkey,
            },
            llm,
            logger: Default::default(),
            p2p: NodeP2PConfig {
                address: self.p2p_address,
                orch_address: info
                    .find_quic()
                    .cloned()
                    .ok_or_else(|| eyre!("No quic address found"))?,
            },
        };

        if self.path.exists() {
            fs::remove_dir_all(&self.path).context("Error when removing a directory for a node")?;
        }
        create_dir_all(&self.path).context("Error when creating a directory for a node")?;

        let path = self.path.join("config.yaml");

        Config::Node(cfg.into())
            .save(&path)
            .context("Failed to save node config")?;
        println!("Node config saved successfully {path:?}");
        println!("Node Public Key: {}", self_key.public_key());

        Ok(())
    }

    async fn load_cluster_info(&self) -> Result<ClusterInfo, Error> {
        let client = orchestrator_client::Client::new(self.orch.clone())
            .context("Failed to create orchestrator client")?;
        client
            .cluster_info()
            .await
            .context("Failed to load cluster info")
    }
}
