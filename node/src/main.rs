use clap::Parser;
use color_eyre::eyre::Result;
use node_config::logging::LoggerConfig;
use std::env;
use tracing_subscriber::EnvFilter;

mod args;

const NODE_PATH_DEFAULT: &str = "./.eve";
const CONFIG_NAME: &str = "config.yaml";

#[tokio::main]
async fn main() -> Result<()> {
    args::Args::parse().execute().await
}

pub fn init_logger(cfg: Option<&LoggerConfig>) {
    let filter = if let Some(cfg) = cfg {
        EnvFilter::from(cfg.filter.clone())
    } else if env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else {
        EnvFilter::builder().parse_lossy("info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .pretty()
        .init();
}
