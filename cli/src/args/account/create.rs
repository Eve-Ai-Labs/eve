use crate::{
    profiles::{Profile, Profiles, DEFAULT_PROFILE},
    utils::{check_name, parse_key},
};
use clap::Parser;
use cli_utils::Prompt;
use color_eyre::eyre::{Context, Result};
use crypto::ed25519::private::PrivateKey;
use tracing::instrument;
use url::Url;

/// Creating a profile. The profile will be saved in the `$HOME/.eve/config.yaml` file.
#[derive(Debug, Parser)]
pub(crate) struct Create {
    /// The profile name. Will be used to access the settings.
    /// It can contain numbers, letters and the characters `-`, `_`
    #[arg(value_parser = check_name, default_value = DEFAULT_PROFILE, verbatim_doc_comment)]
    pub name: String,

    /// The URL of the node
    #[arg(short, long)]
    pub rpc: Url,

    /// The private key in hexadecimal format for the profile or or the path to the file.
    /// If you don't specify it, a new one will be generated.
    #[arg(short = 'k', long = "key", value_parser = parse_key)]
    pub private_key: Option<PrivateKey>,

    #[command(flatten)]
    prompt: Prompt,
}

impl Create {
    #[instrument(level = "debug")]
    pub(crate) async fn execute(self) -> Result<()> {
        let mut profiles = Profiles::load().context("Error when reading profiles")?;

        if self.private_key.is_none()
            && !self
                .prompt
                .prompt_yes("You didn't specify the secret key, should I generate it?")
        {
            return Ok(());
        }

        let Self {
            name,
            rpc,
            private_key: private,
            prompt,
        } = self;

        let private = private.unwrap_or_else(PrivateKey::generate);
        let public = private.public_key();

        println!(
            "Adding a profile:\n\
            name: {name}\n\
            rpc: {rpc}\n\
            public key: {public}\n\
            private key: ***
            "
        );

        // the profile exists
        if profiles.0.contains_key(&name)
            && !prompt.prompt_yes("the profile exists, should overwrite it?")
        {
            // cancel
            return Ok(());
        }

        profiles.0.insert(
            name,
            Profile {
                rpc,
                public,
                private,
                session: Default::default(),
            },
        );
        profiles.save().context("Error when saving profiles")
    }
}
