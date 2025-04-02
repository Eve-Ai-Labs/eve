use crate::{utils::Rpc, OUTPUT_JSON};
use clap::Parser;
use color_eyre::eyre::Result;
use tracing::instrument;

/// Display Nodes
#[derive(Debug, Parser)]
pub(crate) struct List {
    #[command(flatten)]
    rpc: Rpc,

    /// JSON output
    #[arg(short, long)]
    json: bool,
}

impl List {
    #[instrument(level = "debug")]
    pub(crate) async fn execute(&self) -> Result<()> {
        OUTPUT_JSON.set(self.json)?;

        let mut nodes = self.rpc.client().nodes().await?;
        // display the response
        if self.json {
            println!("{}", serde_json::to_string_pretty(&nodes)?);
            return Ok(());
        }

        nodes.sort_by(|a, b| a.peer_id.cmp(&b.peer_id));

        println!("Nodes:");
        for node in &nodes {
            println!("   Peer ID: {}", &node.peer_id);
            println!("   Public Key: {}", &node.key);
            println!(
                "   Address: {}",
                node.address
                    .as_ref()
                    .map(|address| address.to_string())
                    .unwrap_or(" - ".into())
            );
            println!();
        }

        Ok(())
    }
}
