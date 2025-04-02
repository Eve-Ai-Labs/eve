use crate::{
    profiles::{Profiles, DEFAULT_PROFILE},
    utils::check_name,
};
use clap::{arg, Parser};
use color_eyre::eyre::{eyre, ContextCompat as _, Result};
use orchestrator_client::ClientWithKey;
use tracing::instrument;

/// Airdrop some funds to the account
#[derive(Debug, Parser)]
pub struct Airdrop {
    /// The name of the account
    #[arg(value_parser = check_name, default_value = DEFAULT_PROFILE, verbatim_doc_comment)]
    pub profile: String,
}

impl Airdrop {
    #[instrument(level = "debug")]
    pub async fn execute(&self) -> Result<()> {
        let mut profiles = Profiles::load()?;

        let profile = profiles
            .get_mut(&self.profile)
            .with_context(|| eyre!("Profile {:?} not found", self.profile))?;
        let client: ClientWithKey = profile.client()?;
        let balance = client.airdrop().await?;
        println!("Balance: {balance}");
        Ok(())
    }
}
