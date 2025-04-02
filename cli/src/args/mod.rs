pub(crate) mod account;
pub(crate) mod answer;
pub(crate) mod nodes;
pub(crate) mod orch;
pub(crate) mod question;

use crate::args::{account::Account, question::Question};
use answer::Answer;
use clap::Parser;
use color_eyre::eyre::Result;
use nodes::Nodes;
use orch::Orch;

#[derive(Parser)]
pub(crate) enum Args {
    #[command(subcommand, visible_aliases = ["accounts", "profile", "profiles"])]
    Account(Account),

    #[command(name = "ask", visible_aliases = ["send", "question", "run", "request"])]
    Question(Question),

    #[command(visible_aliases = ["answers","result", "results"])]
    Answer(Answer),

    #[command(subcommand)]
    Node(Nodes),

    #[command(subcommand, visible_aliases = ["orchestrator"])]
    Orch(Orch),
}

impl Args {
    pub(crate) async fn execute(self) -> Result<()> {
        match self {
            Self::Account(cmd) => cmd.execute().await,
            Self::Question(mut cmd) => cmd.execute().await,
            Self::Answer(cmd) => cmd.execute().await,
            Self::Node(cmd) => cmd.execute().await,
            Self::Orch(cmd) => cmd.execute().await,
        }
    }
}
