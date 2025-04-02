use crate::utils::Rpc;
use clap::Parser;
use cli_utils::Prompt;
use color_eyre::eyre::{Context, Result};
use crypto::ed25519::public::PublicKey;
use jwt::JwtSecret;
use tracing::instrument;

/// Delete node
#[derive(Debug, Parser)]
pub(crate) struct Delete {
    #[command(flatten)]
    rpc: Rpc,

    /// Jwt Secret
    #[arg(short, long)]
    jwt: JwtSecret,

    /// Node public key
    #[arg(short, long)]
    public_key: PublicKey,

    #[command(flatten)]
    prompt: Prompt,
}

impl Delete {
    #[instrument(level = "debug")]
    pub(crate) async fn execute(self) -> Result<()> {
        println!("Node:");
        println!("Public key: {}", &self.public_key);

        if !self
            .prompt
            .prompt_yes("Are you sure you want to delete a Node?")
        {
            return Ok(());
        }

        self.rpc
            .client()
            .delete_nodes(self.jwt, self.public_key)
            .await
            .context("Couldn't delete node from the list")?;

        println!("The node was successfully deleted");

        Ok(())
    }
}
