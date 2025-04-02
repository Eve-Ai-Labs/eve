use eyre::{Context, Result};
use serde::{de, Deserialize, Deserializer, Serializer};
use std::{fmt::Display, str::FromStr};
use url::Url;

pub fn serialize_url<S>(url: &Url, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(url.as_str())
}

pub fn deserialize_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: Deserializer<'de>,
    D::Error: Display,
{
    let url_str: String = Deserialize::deserialize(deserializer)?;
    let url = Url::from_str(&url_str)
        .context("dsf")
        .map_err(de::Error::custom)?;
    Ok(url)
}
