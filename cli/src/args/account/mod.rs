pub(crate) mod airdrop;
pub(crate) mod balance;
pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod list;

use crate::args::account::{create::Create, delete::Delete, list::List};
use airdrop::Airdrop;
use balance::Balance;
use clap::Parser;
use color_eyre::eyre::Result;

/// Working with accounts
#[derive(Parser)]
pub(crate) enum Account {
    #[command(name = "create", visible_alias = "new")]
    Create(Box<Create>),
    #[command(name = "list")]
    List(List),
    #[command(name = "delete", visible_alias = "remove")]
    Delete(Delete),
    #[command(name = "balance")]
    Balance(Balance),
    #[command(name = "airdrop")]
    Airdrop(Airdrop),
}

impl Account {
    pub(crate) async fn execute(self) -> Result<()> {
        match self {
            Self::Create(cmd) => cmd.execute().await,
            Self::List(cmd) => cmd.execute().await,
            Self::Delete(cmd) => cmd.execute().await,
            Self::Balance(cmd) => cmd.execute().await,
            Self::Airdrop(cmd) => cmd.execute().await,
        }
    }
}
