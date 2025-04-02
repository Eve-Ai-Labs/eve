use crate::{
    display::DisplayAnswer,
    echoln,
    profiles::{Profiles, DEFAULT_SESSION_NAME},
    utils::{check_name, ProfileName},
    OUTPUT_JSON,
};
use clap::Parser;
use color_eyre::eyre::{eyre, ContextCompat, Result};
use tracing::instrument;
use types::ai::query::QueryId;

/// Request a repeat response based on the question ID
#[derive(Debug, Parser)]
pub(crate) struct Answer {
    /// Session name(chat name). Used to store the history
    #[arg(value_parser = check_name, default_value = DEFAULT_SESSION_NAME)]
    session: String,

    /// Request to AI
    #[arg(short, long)]
    query_id: Option<QueryId>,

    #[clap(flatten)]
    profile: ProfileName,

    /// JSON output
    #[arg(short, long)]
    json: bool,
}

impl Answer {
    #[instrument(level = "debug")]
    pub(crate) async fn execute(&self) -> Result<()> {
        if self.json {
            OUTPUT_JSON.set(self.json)?;
        }

        let profile = Profiles::load()?
            .get(&self.profile)
            .cloned()
            .with_context(|| eyre!("Profile `{}` not found", self.profile))?;
        let client = profile.client()?;

        let query_id = match self.query_id {
            Some(id) => id,
            None => profile.session(&self.session).context("The session does not exist. Specify an existing `session` or `request ID` for the request")?,
        };

        echoln!("The request has been sent. QueryID: {query_id}");
        echoln!("Getting a response: ...");

        let result = client.answer(&query_id).await?;

        // display the response
        if self.json {
            let mut result = result.response;
            result.sort_by(|a, b| b.cmp(a));

            println!("{}", serde_json::to_string_pretty(&result)?);
            return Ok(());
        }

        let mut display: DisplayAnswer = result.into();
        display.draw()?;

        Ok(())
    }
}
