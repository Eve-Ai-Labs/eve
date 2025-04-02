use crate::bot;
use clap::Parser;
use eyre::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};
use url::Url;

#[derive(Parser)]
pub struct Args {
    /// The URL of the orchestrator rpc.
    #[arg(short, long, default_value = "http://localhost:1733")]
    pub rpc_url: Url,

    /// The number of parallel questions to ask.
    #[arg(short, long, default_value = "1")]
    pub accounts: u32,

    /// Seconds between each question.
    #[arg(short, long, default_value = "5")]
    pub delay_secs: u64,

    /// The maximum number of history items to keep.
    #[arg(short, long, default_value = "5")]
    pub max_history: u32,
}

impl Args {
    pub(crate) async fn execute(self) -> Result<()> {
        info!("Starting bots...");
        let client = orchestrator_client::Client::new(self.rpc_url)?;
        if let Err(err) = client.status().await {
            error!("Failed to connect to orchestrator: {}", err);
            return Err(err);
        }
        info!("Connected to orchestrator");

        let mut bots = Vec::new();

        for _ in 0..self.accounts {
            let mut bot = bot::Bot::new(client.clone(), self.delay_secs, self.max_history as usize);
            bots.push(tokio::spawn(async move {
                loop {
                    if let Err(err) = bot.run().await {
                        error!("Bot error: {}", err);
                        bot.reset();
                        sleep(Duration::from_secs(5)).await;
                    }
                }
            }));
        }

        for bot in bots {
            bot.await?;
        }

        Ok(())
    }
}
