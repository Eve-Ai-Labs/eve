use crate::utils::Rpc;
use clap::Parser;
use cli_utils::Prompt;
use color_eyre::eyre::{Context, Result};
use crypto::ed25519::public::PublicKey;
use jwt::JwtSecret;
use libp2p::Multiaddr;
use tracing::instrument;
use types::p2p::Peer;

/// Add node
#[derive(Debug, Parser)]
pub(crate) struct Add {
    #[command(flatten)]
    rpc: Rpc,

    /// Jwt Secret.
    #[arg(short, long)]
    jwt: JwtSecret,

    /// Peer public key
    #[arg(short, long)]
    public_key: PublicKey,

    /// Node address.
    /// Format example:
    ///     /ip4/127.0.0.1/udp/10000/quic-v1
    #[arg(short, long, verbatim_doc_comment)]
    address: Option<Multiaddr>,

    #[command(flatten)]
    prompt: Prompt,
}

impl Add {
    #[instrument(level = "debug")]
    pub(crate) async fn execute(self) -> Result<()> {
        let peer = Peer {
            address: self.address,
            public_key: self.public_key,
        };

        println!("NODE");
        println!(
            "Address: {}",
            peer.address
                .as_ref()
                .map(|address| address.to_string())
                .unwrap_or_else(|| "*".into())
        );
        println!("Public key: {}", &peer.public_key);

        if !self
            .prompt
            .prompt_yes("Are you sure you want to add a Node?")
        {
            return Ok(());
        }

        self.rpc
            .client()
            .add_nodes(self.jwt, peer.clone())
            .await
            .context("Couldn't add node to the list")?;

        println!("The nodes was successfully added");

        Ok(())
    }
}
