use crate::init_logger;
use ai::{ollama::Llm, Ai};
use clap::Parser;
use color_eyre::eyre::{ensure, eyre, Context as _, Error, Result};
use crypto::ed25519::{private::PrivateKey, public::PublicKey};
use jwt::JwtSecret;
use multiaddr::Multiaddr;
use node::spawn_node;
use node_config::{
    api::ApiConfig, llm::OllamaConfig, rpc::default_rpc_address, tasks::AiTasksConfig,
};
use orchestrator::{spawn_orchestrator, OrchRequest};
use p2p::Config;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use storage::EveStorage;
use tempdir::TempDir;
use termion::{color, style};
use tokio::{
    sync::{mpsc::Sender, oneshot},
    task::JoinHandle,
};
use tracing::{error, info, instrument};
use types::{cluster::ClusterInfoWithNodes, p2p::Peer, AiModel};
use url::Url;

const LISTENING_QUIC_ADDRESS: &str = "/ip4/127.0.0.1/udp/0/quic-v1";
const LISTENING_WEBRTC_ADDRESS: &str = "/ip4/127.0.0.1/udp/0/webrtc-direct";

/// Launch the test cluster
#[derive(Debug, Parser)]
pub(crate) struct TestRun {
    /// The path to the database storage.
    /// Optional parameter.
    /// If it is not specified, a temporary folder will be used.
    /// The temporary folder will be deleted after the node is shut down.
    #[arg()]
    path: Option<PathBuf>,

    /// IP address with a port for Orchestrator.
    #[arg(short, long, default_value_t = default_rpc_address())]
    rpc: SocketAddr,

    /// Counts of nodes to start. Default is 3
    #[arg(short, long, default_value_t = 3)]
    node_count: u16,

    /// URL to the ollama. Default is http://localhost:11434
    #[arg(short, long, default_value = "http://localhost:11434")]
    ollama_url: Url,

    /// Model to use for the ollama
    #[arg(short, long, default_value_t = AiModel::DeepseekR1_8b)]
    ai_model: AiModel,
}

impl TestRun {
    #[instrument(level = "info")]
    pub(crate) async fn execute(self) -> Result<()> {
        init_logger(None);
        info!("Running test node");

        let (_, store) = storage(self.path.as_ref())?;

        let ai = Arc::new(
            Llm::new(&OllamaConfig {
                url: self.ollama_url,
                model: self.ai_model.to_string(),
                ..Default::default()
            })
            .await
            .unwrap(),
        );

        let orch_p2p = orch_p2p().await?;
        info!("Orchestrator key: {:?}", orch_p2p.key.public_key());

        let nodes_p2p = nodes_p2p(self.node_count, orch_p2p.key.public_key()).await?;

        let orch_key = orch_p2p.key.clone();

        let peers = peers(&nodes_p2p).await;
        info!("Network: {:?}", peers);

        let mut handlers = Vec::with_capacity(nodes_p2p.len());

        let (orch_handler, api, jwt) =
            spawn_orch(ai.clone(), store, self.rpc, orch_p2p, peers).await?;
        handlers.extend(orch_handler);

        let orch_addr = wait_for_orch_address(api.clone()).await?;
        info!("Orchestrator address: {:?}", orch_addr);

        for node in nodes_p2p {
            api.send(OrchRequest::AddNode {
                address: None,
                public_key: node.key.public_key(),
                tx: oneshot::channel().0,
            })
            .await?;

            let hndl =
                start_node(ai.clone(), orch_key.public_key(), node, orch_addr.clone()).await?;
            handlers.extend(hndl);
        }

        let hndl = print_cluster_info(api, orch_addr, self.rpc, jwt).await;
        handlers.push(hndl);

        wait_for(handlers).await?;
        Ok(())
    }
}

async fn print_cluster_info(
    api: Sender<OrchRequest>,
    orch_addr: Multiaddr,
    rpc: SocketAddr,
    jwt: JwtSecret,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            match cluster_info(&api).await {
                Ok(info) => {
                    let mut lines = Vec::new();
                    lines.push(format!(
                        "{}Orchestrator address:{} {}",
                        color::Fg(color::Blue),
                        style::Reset,
                        orch_addr
                    ));

                    lines.push(format!(
                        "{}WebRTC certhash path:{} {:?}",
                        color::Fg(color::Blue),
                        style::Reset,
                        info.cluster_info.webrtc_certhash
                    ));

                    lines.push(format!(
                        "{}All Orchestrator addresses:{} {:?}",
                        color::Fg(color::Blue),
                        style::Reset,
                        info.cluster_info.orch_address
                    ));

                    for node in info.nodes.values() {
                        let status = if node.is_connected() {
                            format!(
                                "{}Connected({}){}",
                                color::Fg(color::Green),
                                node.connected,
                                style::Reset
                            )
                        } else {
                            format!("{}Disconnected{}", color::Fg(color::Red), style::Reset)
                        };

                        lines.push(format!(
                            "{}Node address:{} {:?} - {}",
                            color::Fg(color::Blue),
                            style::Reset,
                            node.address,
                            status
                        ));
                    }

                    lines.push(format!(
                        "{}RPC address:{} {}",
                        color::Fg(color::Blue),
                        style::Reset,
                        rpc
                    ));

                    lines.push(format!(
                        "{}JWT:{} {}",
                        color::Fg(color::Blue),
                        style::Reset,
                        jwt
                    ));

                    let max_width = lines
                        .iter()
                        .map(|line| line.chars().filter(|c| !c.is_control()).count())
                        .max()
                        .unwrap_or(0);

                    let horizontal_border = format!("{}╔{}", style::Reset, "═".repeat(10));
                    let bottom_border = format!("╚{}", "═".repeat(10));

                    let mut framed_output = String::new();
                    framed_output.push_str(&horizontal_border);
                    framed_output.push('\n');

                    for line in lines {
                        let visible_length = line.chars().filter(|c| !c.is_control()).count();
                        let padding = max_width - visible_length;
                        framed_output.push_str(&format!("║ {}{}\n", line, " ".repeat(padding)));
                    }

                    framed_output.push_str(&bottom_border);

                    info!("Cluster info:\n{}", framed_output);
                }
                Err(err) => {
                    error!(%err);
                }
            }
        }
    })
}

async fn cluster_info(api: &Sender<OrchRequest>) -> Result<ClusterInfoWithNodes, Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    api.send(OrchRequest::ClusterInfo { tx })
        .await
        .map_err(|_| eyre!("Failed to send request to orchestrator"))?;
    let info = rx
        .await
        .map_err(|_| eyre!("Failed to receive response from orchestrator"))?;
    Ok(info)
}

async fn wait_for_orch_address(api: Sender<OrchRequest>) -> Result<Multiaddr, Error> {
    loop {
        let info = cluster_info(&api).await?;
        if info.cluster_info.orch_address.is_empty() {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        } else {
            return Ok(info.cluster_info.orch_address[0].clone());
        }
    }
}

async fn wait_for(mut handlers: Vec<JoinHandle<()>>) -> Result<()> {
    'task_loop: loop {
        for hndl in handlers.iter_mut() {
            if hndl.is_finished() {
                break 'task_loop;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    for hndl in handlers {
        hndl.abort();
    }

    Ok(())
}

async fn spawn_orch<AI: Ai + Send + Sync + 'static>(
    ai: Arc<AI>,
    store: Arc<EveStorage>,
    rpc: SocketAddr,
    orch: Orch,
    peers: Vec<Peer>,
) -> Result<(Vec<JoinHandle<()>>, Sender<OrchRequest>, JwtSecret)> {
    let (api_tx, api_rx) = tokio::sync::mpsc::channel(100);

    let cfg = ApiConfig::default();
    let jwt = cfg.jwt;
    info!("JWT: {}", jwt);

    let api_handles = orchestrator_api::run(rpc, api_tx.clone(), store.clone(), cfg).await;

    let cfg = AiTasksConfig {
        whitelist: peers,
        replication_factor: 3,
        task_timeout_secs: 60,
    };

    let orch_handles = spawn_orchestrator(
        store,
        api_rx,
        (orch.p2p_sender, orch.p2p_receiver),
        ai,
        orch.key,
        &cfg,
    )
    .await?;
    Ok((
        vec![
            api_handles,
            orch.p2p.handler,
            orch_handles.ev,
            orch_handles.orch,
        ],
        api_tx,
        jwt,
    ))
}

async fn start_node<AI: Ai + Send + Sync + 'static>(
    ai: Arc<AI>,
    orch_key: PublicKey,
    node: Node,
    orch_address: Multiaddr,
) -> Result<Vec<JoinHandle<()>>> {
    let node_handler = spawn_node(
        node.p2p_sender,
        node.p2p_receiver,
        ai,
        orch_key,
        node.key,
        orch_address,
    )
    .await
    .context("Failed to spawn node runtime")?;

    Ok(vec![node_handler.handler, node.p2p.handler])
}

fn storage(path: Option<&PathBuf>) -> Result<(Option<TempDir>, Arc<EveStorage>)> {
    if let Some(path) = path {
        storage_with_path(path).map(|storage| (None, storage))
    } else {
        storage_with_tmp().map(|(temp_path, storage)| (Some(temp_path), storage))
    }
}

fn storage_with_tmp() -> Result<(TempDir, Arc<EveStorage>)> {
    let tmp_dir = TempDir::new("rocksdb")?;
    let path = tmp_dir.path();
    let store = EveStorage::new(path, &Default::default())?;
    Ok((tmp_dir, Arc::new(store)))
}

fn storage_with_path(path: &Path) -> Result<Arc<EveStorage>> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .with_context(|| eyre!("Couldn't create a directory {path:?} for the node"))?;
    }
    ensure!(path.is_dir(), "Path is not a directory {path:?}");

    let store = EveStorage::new(path, &Default::default())?;
    Ok(Arc::new(store))
}

async fn nodes_p2p(node_count: u16, orch_key: PublicKey) -> Result<Vec<Node>> {
    let mut nodes = Vec::new();

    for _ in 0..node_count {
        let key = PrivateKey::generate();
        let (p2p, p2p_receiver, p2p_sender) = p2p::spawn(
            key.clone(),
            orch_key,
            vec![],
            &[LISTENING_QUIC_ADDRESS.parse()?],
            Config::default(),
        )
        .await?;

        nodes.push(Node {
            key,
            p2p_sender,
            p2p_receiver,
            p2p,
        });
    }

    Ok(nodes)
}

async fn peers(nodes: &[Node]) -> Vec<Peer> {
    let mut peers = Vec::new();

    for node in nodes {
        peers.push(Peer {
            address: None,
            public_key: node.key.public_key(),
        });
    }

    peers
}

struct Node {
    key: PrivateKey,
    p2p_sender: node::ToP2P,
    p2p_receiver: node::FromP2P,
    p2p: p2p::P2PHandler,
}

async fn orch_p2p() -> Result<Orch> {
    let key = PrivateKey::generate();
    let (p2p, p2p_receiver, p2p_sender) = p2p::spawn(
        key.clone(),
        key.public_key(),
        vec![],
        &[
            LISTENING_QUIC_ADDRESS.parse()?,
            LISTENING_WEBRTC_ADDRESS.parse()?,
        ],
        Config::default(),
    )
    .await?;

    Ok(Orch {
        key,
        p2p_sender,
        p2p_receiver,
        p2p,
    })
}

struct Orch {
    key: PrivateKey,
    p2p_sender: orchestrator::ToP2P,
    p2p_receiver: orchestrator::FromP2P,
    p2p: p2p::P2PHandler,
}
