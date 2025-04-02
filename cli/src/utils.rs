use crate::profiles::{Profiles, DEFAULT_PROFILE};
use clap::Parser;
use color_eyre::eyre::{ensure, ContextCompat, Result};
use crypto::ed25519::private::PrivateKey;
use orchestrator_client::Client;
use std::{fmt::Display, fs, str::FromStr};
use url::Url;

const MAX_NAME_LENGTH: usize = 50;

pub(crate) fn check_name(name: &str) -> Result<String> {
    ensure!(
        name.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
        "Forbidden characters in the name"
    );
    ensure!(
        name.len() < MAX_NAME_LENGTH,
        "The maximum allowed length of the name is {MAX_NAME_LENGTH} characters."
    );

    Ok(name.into())
}

pub(crate) fn parse_key(key: &str) -> Result<PrivateKey> {
    let key = fs::read_to_string(key).unwrap_or(key.to_string());
    PrivateKey::from_str(key.trim())
}

#[derive(Debug, Parser)]
pub(crate) struct Rpc {
    /// The address to connect to the orchestra or profile name
    #[arg(value_parser=parse_rpc, default_value = DEFAULT_PROFILE)]
    pub rpc: Url,
}

impl Rpc {
    pub(crate) fn client(&self) -> Client {
        Client::new(self.rpc.clone()).expect("Never")
    }
}

pub(crate) fn parse_rpc(value: &str) -> Result<Url> {
    Url::parse(value)
        .ok()
        .or_else(|| {
            Profiles::load()
                .ok()?
                .get(value)
                .map(|profile| profile.rpc.clone())
        })
        .context("Invalid value")
}

#[derive(Debug, Parser)]
pub(crate) struct ProfileName {
    /// The request will be made on behalf of this profile.
    #[arg(short, long, value_parser = check_name, default_value = DEFAULT_PROFILE, verbatim_doc_comment)]
    profile: String,
}

impl Display for ProfileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.profile)
    }
}
