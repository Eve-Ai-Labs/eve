use crate::utils::Rpc;
use clap::Parser;
use color_eyre::eyre::Result;
use tracing::instrument;

/// Working with Orchestration
#[derive(Parser)]
pub(crate) enum Orch {
    #[command(name = "info")]
    Info(Info),
    Metrics(Metrics),
}

impl Orch {
    pub(crate) async fn execute(self) -> Result<()> {
        match self {
            Self::Info(cmd) => cmd.execute().await,
            Self::Metrics(cmd) => cmd.execute().await,
        }
    }
}

/// Orchestration info
#[derive(Debug, Parser)]
pub(crate) struct Info {
    #[command(flatten)]
    rpc: Rpc,
}

impl Info {
    #[instrument(level = "debug")]
    pub(crate) async fn execute(self) -> Result<()> {
        let info = self.rpc.client().cluster_info().await?;
        println!("cluster info: {info:?}");

        Ok(())
    }
}

/// Orchestration metrics
#[derive(Debug, Parser)]
pub(crate) struct Metrics {
    #[command(flatten)]
    rpc: Rpc,
}

impl Metrics {
    #[instrument(level = "debug")]
    pub(crate) async fn execute(self) -> Result<()> {
        let metrics = self.rpc.client().metrics().await?;
        println!("cluster metrics: {metrics:?}");

        Ok(())
    }
}
