mod args;
mod display;
mod profiles;
mod utils;

use clap::Parser;
use color_eyre::eyre::Result;
use home::home_dir;
use std::{fs, path::PathBuf, sync::LazyLock};
use tokio::sync::OnceCell;
use tracing_subscriber::EnvFilter;

pub(crate) static EVE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let home = home_dir().expect("Could not get home directory");
    let eve_path = home.join(".eve");
    if !eve_path.exists() {
        fs::create_dir_all(&eve_path).expect("Error when creating $HOME/.eve");
    }
    eve_path
});
pub(crate) static EVE_CONFIG_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| EVE_PATH.join("config.yaml"));
static OUTPUT_JSON: OnceCell<bool> = OnceCell::const_new();

#[macro_export]
macro_rules! echoln {
    () => {
        if !OUTPUT_JSON.cloned().unwrap_or_default(){
            println!()
        }
    };
    ($($arg:tt)*) => {{
        if !OUTPUT_JSON.get().cloned().unwrap_or_default(){
            println!($($arg)*)
        }
    }};
}

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
