pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod list;

use clap::Parser;
use color_eyre::eyre::Result;
use create::Add;
use delete::Delete;
use list::List;

/// Working with Nodes
#[derive(Parser)]
pub(crate) enum Nodes {
    #[command(name = "list")]
    List(Box<List>),

    #[command()]
    Add(Box<Add>),

    #[command(name = "delete", visible_alias = "remove")]
    Delete(Box<Delete>),
}

impl Nodes {
    pub(crate) async fn execute(self) -> Result<()> {
        match self {
            Self::Add(cmd) => cmd.execute().await,
            Self::List(cmd) => cmd.execute().await,
            Self::Delete(cmd) => cmd.execute().await,
        }
    }
}
