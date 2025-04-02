use crate::{init_logger, CONFIG_NAME, NODE_PATH_DEFAULT};
use ai::{ollama::Llm, Ai};
use clap::Parser;
use color_eyre::eyre::{ensure, Context, Result};
use node::spawn_node;
use node_config::{load_config, node::NodeConfig, orch::OrchConfig};
use orchestrator::spawn_orchestrator;
use p2p::Config;
use std::{path::PathBuf, sync::Arc};
use storage::EveStorage;

/// Launch the node
#[derive(Debug, Parser)]
pub(crate) struct Run {
    /// The directory for the node
    #[arg(default_value = NODE_PATH_DEFAULT)]
    path: PathBuf,
}

impl Run {
    pub(crate) async fn execute(self) -> Result<()> {
        let mut config_path = self.path.clone();
        if config_path.is_dir() {
            config_path = config_path.join(CONFIG_NAME);
        }

        ensure!(
            config_path.exists(),
            "Configuration file not {config_path:?} found"
        );

        let cfg = load_config(config_path).context("Failed to load configuration")?;

        init_logger(Some(cfg.logger()));

        let llm = Llm::new(cfg.llm()).await?;
        let ai = Arc::new(llm);

        match cfg {
            node_config::Config::Orch(cfg) => start_orchestrator(*cfg, ai)
                .await
                .context("Failed to start orchestrator"),
            node_config::Config::Node(cfg) => {
                start_node(*cfg, ai).await.context("Failed to start node")
            }
        }
    }
}

async fn start_orchestrator<AI: Ai + Send + Sync + 'static>(
    cfg: OrchConfig,
    ai: Arc<AI>,
) -> Result<()> {
    let store = Arc::new(
        EveStorage::new(&cfg.db.path, &cfg.db.rocksdb).context("Failed to create storage")?,
    );

    let (api_tx, api_rx) = tokio::sync::mpsc::channel(100);
    let mut api_handles =
        orchestrator_api::run(cfg.rpc.address, api_tx, store.clone(), cfg.api).await;

    let (mut p2p, p2p_rx, p2p_tx) = p2p::spawn(
        cfg.base.key.clone(),
        cfg.base.key.public_key(),
        cfg.p2p.address.clone(),
        &cfg.p2p.address,
        Config::default(),
    )
    .await?;

    let mut orch_handles = spawn_orchestrator(
        store,
        api_rx,
        (p2p_tx, p2p_rx),
        ai,
        cfg.base.key.clone(),
        &cfg.ai_tasks,
    )
    .await
    .context("Failed to spawn orchestrator runtime")?;

    let result = tokio::select! {
        result = orch_handles.wait() => result,
        result = &mut p2p.handler => result.context("running orchestrator"),
        result = &mut api_handles => result.context("running orchestrator")
    };

    orch_handles.abort();
    api_handles.abort();
    p2p.handler.abort();

    result
}

async fn start_node<AI: Ai + Send + Sync + 'static>(cfg: NodeConfig, ai: Arc<AI>) -> Result<()> {
    let (mut p2p, p2p_rx, p2p_tx) = p2p::spawn(
        cfg.base.key.clone(),
        cfg.base.orch_pub_key,
        vec![cfg.p2p.orch_address.clone()],
        &cfg.p2p.address,
        Config::default(),
    )
    .await?;

    let mut node = spawn_node(
        p2p_tx,
        p2p_rx,
        ai,
        cfg.base.orch_pub_key,
        cfg.base.key.clone(),
        cfg.p2p.orch_address.clone(),
    )
    .await
    .context("Failed to spawn node runtime")?;

    let result = tokio::select! {
        result = &mut node.handler => result.context("running node"),
        result = &mut p2p.handler => result.context("running node")
    };

    node.handler.abort();
    p2p.handler.abort();

    result
}
