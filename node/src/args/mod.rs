pub(crate) mod config;
pub(crate) mod init;
pub(crate) mod run;
pub(crate) mod test;

use crate::args::{init::Init, run::Run};
use clap::Parser;
use color_eyre::eyre::Result;
use config::CfgNode;
use test::TestRun;

#[derive(Debug, Parser)]
pub(crate) enum Args {
    #[command(name = "init")]
    Init(Box<Init>),

    #[command(name = "cfg-node")]
    CfgNode(Box<CfgNode>),

    #[command(name = "run")]
    Run(Run),

    #[command(name = "test-run")]
    TestRun(TestRun),
}

impl Args {
    pub(crate) async fn execute(self) -> Result<()> {
        match self {
            Self::Init(cmd) => cmd.execute().await,
            Self::Run(cmd) => cmd.execute().await,
            Self::TestRun(cmd) => cmd.execute().await,
            Self::CfgNode(cmd) => cmd.execute().await,
        }
    }
}
