use clap::Parser as _;
use eyre::Result;
use tracing_subscriber::EnvFilter;

mod args;
mod bot;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .pretty()
        .init();

    args::Args::parse().execute().await
}
