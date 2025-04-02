use crate::init_logger;
use clap::Parser;
use color_eyre::eyre::{Context, Result};
use crypto::ed25519::private::PrivateKey;
use jwt::JwtSecret;
use multiaddr::Multiaddr;
use node_config::{
    api::ApiConfig,
    base::BaseConfig,
    llm::OllamaConfig,
    node::NodeConfig,
    orch::OrchConfig,
    p2p::{NodeP2PConfig, OrchP2PConfig},
    rpc::{default_rpc_address, RpcConfig},
    tasks::AiTasksConfig,
    Config,
};
use std::{
    fs::{self, create_dir_all},
    net::SocketAddr,
    path::PathBuf,
};
use types::p2p::Peer;
use url::Url;

const NODE_PATH_DEFAULT: &str = "./eve";

/// Initialize the configuration for the node
#[derive(Debug, Parser)]
pub(crate) struct Init {
    /// The directory for the node. Default is ./eve
    #[arg(default_value = NODE_PATH_DEFAULT)]
    path: PathBuf,

    /// URL to the ollama. Default is http://localhost:11434
    #[arg(short = 'l', long, default_value = "http://localhost:11434")]
    ollama_url: Url,

    /// Model to use for the ollama. Default is deepseek-r1:1.5b
    #[arg(short, long, default_value = "deepseek-r1:1.5b")]
    ai_model: String,

    /// Node multiaddress
    #[arg(short, long)]
    nodes: Vec<Multiaddr>,

    /// Orchestrator quic multiaddress
    /// Example: /ip4/127.0.0.1/udp/0/quic-v1
    #[arg(long, verbatim_doc_comment)]
    quic: Multiaddr,

    /// Orchestrator webrtc multiaddress
    /// Example: /ip4/127.0.0.1/udp/9903/webrtc-direct
    #[arg(long, verbatim_doc_comment)]
    webrtc: Multiaddr,

    /// Jwt Secret.
    #[arg(short, long)]
    jwt: Option<JwtSecret>,

    /// IP address with a port for Orchestrator.
    #[arg(short, long, default_value_t = default_rpc_address())]
    rpc: SocketAddr,
}

impl Init {
    pub(crate) async fn execute(self) -> Result<()> {
        init_logger(None);

        let path = &self.path;
        let rpc = self.rpc;
        let orch_quic_addr = self.quic.clone();
        let orch_webrtc_addr = self.webrtc.clone();

        let orch_key = PrivateKey::generate();
        let llm = self.llm_config();

        let (peers, keys) = self.peers();

        for (index, (peer, key)) in peers.iter().zip(keys.iter()).enumerate() {
            let cfg = NodeConfig {
                base: BaseConfig {
                    key: key.clone(),
                    pub_key: key.public_key(),
                    orch_pub_key: orch_key.public_key(),
                },
                llm: llm.clone(),
                logger: Default::default(),
                p2p: NodeP2PConfig {
                    address: vec![peer.address.clone().unwrap()],
                    orch_address: orch_quic_addr.clone(),
                },
            };

            let dir = path.join(format!("node_{}", index));
            if dir.exists() {
                fs::remove_dir_all(&dir).context("Error when removing a directory for a node")?;
            }

            create_dir_all(&dir).context("Error when creating a directory for a node")?;
            let path = dir.join("config.yaml");
            Config::Node(cfg.into())
                .save(&path)
                .context("Failed to save node config")?;
            println!("Node {index} config saved successfully {path:?}");
        }

        let mut orch_cfg = OrchConfig {
            base: BaseConfig {
                key: orch_key.clone(),
                pub_key: orch_key.public_key(),
                orch_pub_key: orch_key.public_key(),
            },
            llm,
            ai_tasks: AiTasksConfig {
                whitelist: peers,
                ..Default::default()
            },
            logger: Default::default(),
            db: Default::default(),
            rpc: RpcConfig { address: rpc },
            api: ApiConfig::default(),
            p2p: OrchP2PConfig {
                address: vec![orch_quic_addr, orch_webrtc_addr],
            },
        };
        if let Some(jwt) = self.jwt {
            orch_cfg.api.jwt = jwt;
        } else {
            println!(
                "Warning: JWT secret is not provided. Orchestrator API will be unauthenticated."
            );
        }

        let dir = path.join("orch");
        if dir.exists() {
            fs::remove_dir_all(&dir).context("Error when removing a directory for a node")?;
        }
        create_dir_all(&dir).context("Error when creating a directory for a node")?;
        let orch_path = dir.join("config.yaml");

        Config::Orch(orch_cfg.into())
            .save(&orch_path)
            .context("Failed to save orchestrator config")?;
        println!("Orchestrator config saved successfully {orch_path:?}");
        Ok(())
    }

    fn peers(&self) -> (Vec<Peer>, Vec<PrivateKey>) {
        self.nodes
            .iter()
            .map(|node| {
                let private_key = PrivateKey::generate();
                (
                    Peer {
                        address: Some(node.clone()),
                        public_key: private_key.public_key(),
                    },
                    private_key,
                )
            })
            .fold(
                (Vec::new(), Vec::new()),
                |(mut peers, mut keys), (peer, key)| {
                    peers.push(peer);
                    keys.push(key);
                    (peers, keys)
                },
            )
    }

    fn llm_config(&self) -> OllamaConfig {
        OllamaConfig {
            url: self.ollama_url.clone(),
            model: self.ai_model.clone(),
            ..Default::default()
        }
    }
}
