use crate::{profiles::Profiles, EVE_CONFIG_PATH};
use clap::Parser;
use color_eyre::eyre::{eyre, Context, Result};
use tracing::instrument;

/// Display Profiles. The profiles are stored in the file `$HOME/.eve/config.yaml'.
#[derive(Debug, Parser)]
pub(crate) struct List;

impl List {
    #[instrument(level = "debug")]
    pub(crate) async fn execute(&self) -> Result<()> {
        let path = &*EVE_CONFIG_PATH;
        println!("Profiles:");

        if !path.exists() {
            println!("No profiles found");
            return Ok(());
        }

        let profiles = Profiles::load().with_context(|| eyre!("Error when loading profiles"))?;
        println!("{profiles}");

        Ok(())
    }
}
