use crate::{profiles::Profiles, utils::check_name};
use clap::Parser;
use cli_utils::Prompt;
use color_eyre::eyre::{Context, Result};
use tracing::instrument;

/// Delete a profile. The profile will be saved in the `$HOME/.eve/config.yaml` file.
#[derive(Debug, Parser)]
pub(crate) struct Delete {
    /// The name of the profile to delete
    #[arg(value_parser = check_name, verbatim_doc_comment)]
    pub name: String,

    #[command(flatten)]
    prompt: Prompt,
}

impl Delete {
    #[instrument(level = "debug")]
    pub(crate) async fn execute(&self) -> Result<()> {
        let mut profiles = Profiles::load().context("Error when reading profiles")?;

        let Some(profile) = profiles.0.get(&self.name) else {
            println!("A profile with {} name was not found.", &self.name);
            return Ok(());
        };

        if !self.prompt.prompt_yes(format!(
            "Are you sure you want to delete profile {:?}?\n\
            {profile}",
            &self.name
        )) {
            // cancel
            return Ok(());
        }

        profiles.0.remove(&self.name);
        profiles.save().context("Error when saving profiles")
    }
}
