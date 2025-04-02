use crate::EVE_CONFIG_PATH;
use color_eyre::eyre::{eyre, Context, ContextCompat, Result};
use crypto::ed25519::{private::PrivateKey, public::PublicKey};
use orchestrator_client::ClientWithKey;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    fs::File,
    str::FromStr,
};
use tracing::{debug, instrument};
use types::ai::query::QueryId;
use url::Url;

pub(crate) const DEFAULT_PROFILE: &str = "default";
pub(crate) const DEFAULT_SESSION_NAME: &str = "default";

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Profiles(pub HashMap<String, Profile>);

impl Profiles {
    pub(crate) fn load() -> Result<Self> {
        let path = &*EVE_CONFIG_PATH;
        if !path.exists() {
            return Ok(Self::default());
        }
        let file = File::open(path).with_context(|| eyre!("Error when opening {path:?}"))?;
        serde_yaml::from_reader(file).with_context(|| {
            eyre!("Error when parsing {path:?}. The format is different. Try deleting the config.")
        })
    }

    pub fn get<S: ToString>(&self, profile: S) -> Option<&Profile> {
        self.0.get(&profile.to_string())
    }

    pub fn get_mut<S: ToString>(&mut self, profile: S) -> Option<&mut Profile> {
        self.0.get_mut(&profile.to_string())
    }

    #[instrument(level = "debug", skip_all)]
    pub(crate) fn save(&self) -> Result<()> {
        let path = &*EVE_CONFIG_PATH;
        let file = File::create(path).with_context(|| eyre!("Error when opening {path:?}"))?;
        debug!("Saving profiles: \n{self}");
        serde_yaml::to_writer(file, self).with_context(|| eyre!("Error when saving {path:?}"))
    }

    pub(crate) fn set_and_save_session<P, S>(
        &mut self,
        profile: P,
        session: S,
        query: QueryId,
    ) -> Result<()>
    where
        P: Display,
        S: Display,
    {
        self.get_mut(&profile)
            .with_context(|| eyre!("Profile {profile} not found"))?
            .set_session(session, query);
        self.save()
    }
}

impl Display for Profiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = self.0.iter().collect::<Vec<(_, _)>>();
        s.sort_by(|a, b| a.0.cmp(b.0));

        s.iter().try_for_each(|(name, v)| {
            writeln!(f, "{name:?}:")?;
            writeln!(f, "{v:#?}")
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Profile {
    #[serde(serialize_with = "serialize_url", deserialize_with = "deserialize_url")]
    pub rpc: Url,
    pub public: PublicKey,
    pub private: PrivateKey,
    pub session: Session,
}

impl Profile {
    pub(crate) fn client(&self) -> Result<ClientWithKey> {
        ClientWithKey::new(self.private.clone(), self.rpc.clone())
    }

    pub(crate) fn session<S: ToString>(&self, name: S) -> Option<QueryId> {
        self.session.0.get(&name.to_string()).cloned()
    }

    pub(crate) fn set_session<S: ToString>(&mut self, name: S, query: QueryId) {
        self.session.0.insert(name.to_string(), query);
    }
}

impl Debug for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{self}")
    }
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            rpc,
            public,
            private,
            session,
        } = self;
        let mut session = session
            .to_string()
            .lines()
            .map(|v| format!("   {v}"))
            .collect::<Vec<_>>()
            .join("\n");
        if !session.is_empty() {
            session = format!("session: \n{session}");
        }
        writeln!(
            f,
            "rpc: {rpc}\n\
            public key: {public}\n\
            private key: {private}\n\
            {session}"
        )
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct Session(pub(crate) HashMap<String, QueryId>);

impl Display for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = self.0.iter().collect::<Vec<(_, _)>>();
        s.sort_by(|a, b| a.0.cmp(b.0));

        s.iter()
            .try_for_each(|(name, v)| writeln!(f, "{name}: {v}"))
    }
}

fn serialize_url<S>(url: &Url, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(url.as_str())
}

fn deserialize_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
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
