use crypto::ed25519::{private::PrivateKey, public::PublicKey};
use eyre::{bail, eyre, Context, ContextCompat, Result};
use jwt::JwtSecret;
use reqwest::{
    header::{AUTHORIZATION, CACHE_CONTROL},
    Url,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "time")]
use std::time::Duration;
use std::{fmt::Display, ops::Deref, sync::Arc};
use tracing::{debug, instrument};
use types::{
    ai::{
        models::AiDownloadModel,
        query::{Query, QueryId},
        request::{AiRequest, History},
    },
    cluster::{ClusterInfo, Node, NodeInfo},
    p2p::Peer,
};

#[derive(Clone)]
pub struct Client {
    rpc: Arc<Url>,
    client: reqwest::Client,
}

impl Client {
    pub fn new<U>(url: U) -> Result<Self>
    where
        U: TryInto<Url>,
        <U as TryInto<Url>>::Error: std::fmt::Debug,
    {
        let rpc = url.try_into().map_err(|err| eyre!("{err:?}"))?;
        let mut default_headers = reqwest::header::HeaderMap::new();
        default_headers.insert(CACHE_CONTROL, "no-cache".parse().unwrap());
        let client = reqwest::Client::builder()
            .default_headers(default_headers)
            .build()
            .map_err(|err| eyre!("{err:?}"))?;

        Ok(Self {
            rpc: Arc::new(rpc),
            client,
        })
    }

    pub async fn send<S, T, R>(&self, suff: S, request: &T) -> Result<R>
    where
        S: AsRef<str>,
        T: Serialize,
        R: for<'de> serde::Deserialize<'de>,
    {
        let url = self.rpc.join(suff.as_ref())?;
        self.client
            .post(url)
            .json(request)
            .send()
            .await
            .context("Request error")?
            .error_for_status()?
            .json()
            .await
            .context("Couldn't get a response")
    }

    pub async fn get<S, R>(&self, suff: S) -> Result<R>
    where
        S: AsRef<str>,
        R: for<'de> serde::Deserialize<'de>,
    {
        self.client
            .get(self.rpc.join(suff.as_ref())?)
            .send()
            .await
            .context("Request error")?
            .error_for_status()?
            .json()
            .await
            .context("Couldn't get a response")
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn history<S: Display>(&mut self, query: S) -> Result<Vec<History>> {
        debug!("last query: {query}");

        self.get(format!("/history/{query}"))
            .await
            .context("Error when receiving the history")
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn status(&self) -> Result<ApiStatus> {
        debug!("status");
        self.get("").await
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn cost(&self, count: u64) -> Result<u64> {
        self.status()
            .await?
            .cost
            .checked_mul(count)
            .context("Too much")
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn account(&self, account: &PublicKey) -> Result<AccountInfo> {
        self.get(format!("/account/{account}"))
            .await
            .context("Error when receiving the account info")
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn balance(&self, account: &PublicKey) -> Result<u64> {
        self.account(account)
            .await
            .map(|acc| acc.balance)
            .context("Error when receiving the balance")
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn airdrop(&self, account: &PublicKey) -> Result<u64> {
        let info: AccountInfo = self
            .send(format!("/account/airdrop/{account}"), &())
            .await
            .context("Error when receiving the account info")?;
        Ok(info.balance)
    }

    pub async fn query<S: ToString>(
        &self,
        query: S,
        history: Vec<History>,
        key: &PrivateKey,
    ) -> Result<QueryId> {
        self.send(
            "/query",
            &AiRequest::new(query.to_string(), history, key.public_key()).sign(key)?,
        )
        .await
    }

    pub async fn answer(&self, query_id: &QueryId) -> Result<Query> {
        self.get(format!("/answer/{query_id}")).await
    }

    #[cfg(feature = "time")]
    #[instrument(level = "debug", skip_all)]
    pub async fn answer_wait(
        &self,
        query_id: &QueryId,
        duration: Option<Duration>,
    ) -> Result<Query> {
        use eyre::bail;

        let mut duration = duration.unwrap_or_else(|| Duration::from_secs(300));
        let sleep = Duration::from_secs(1);

        loop {
            let query = self.answer(query_id).await?;
            debug!("{query:#?}");

            if query.is_complete() {
                debug!("complete");
                return Ok(query);
            }

            duration -= sleep;
            if duration.is_zero() {
                bail!("The waiting time has expired");
            }

            debug!("sleep");
            tokio::time::sleep(sleep).await;
        }
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn cluster_info(&self) -> Result<ClusterInfo> {
        self.get("/info")
            .await
            .context("Error when receiving the info")
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn nodes(&self) -> Result<Vec<Node>> {
        self.get("/nodes")
            .await
            .context("Error when receiving the nodes")
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn metrics(&self) -> Result<ClusterInfo> {
        self.get("/metrics")
            .await
            .context("Error when receiving the metrics")
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn node(&self, pubkey: PublicKey) -> Result<Option<NodeInfo>> {
        self.get(&format!("/nodes/{}", pubkey))
            .await
            .context("Error when receiving the node")
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn add_nodes(&self, jwt: JwtSecret, node: Peer) -> Result<()> {
        let response = reqwest::Client::new()
            .put(self.rpc.join("/nodes/action")?)
            .header(
                AUTHORIZATION,
                jwt.to_bearer().unwrap().to_str().unwrap().to_string(),
            )
            .json(&node)
            .send()
            .await
            .context("Request error")?;
        let status = response.status();
        if !status.is_success() {
            bail!("{status}: {}", response.text().await.unwrap_or_default());
        }
        response.error_for_status()?;

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn delete_nodes(&self, jwt: JwtSecret, node: PublicKey) -> Result<()> {
        let response = reqwest::Client::new()
            .delete(self.rpc.join("/nodes/action")?)
            .header(
                AUTHORIZATION,
                jwt.to_bearer().unwrap().to_str().unwrap().to_string(),
            )
            .json(&node)
            .send()
            .await
            .context("Request error")?;

        let status = response.status();
        if !status.is_success() {
            bail!("{status}: {}", response.text().await.unwrap_or_default());
        }
        response.error_for_status()?;

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn get_ai(&self) -> Result<AiDownloadModel> {
        self.get("ai").await.context("Error when getting the AI")
    }
}

pub struct ClientWithKey {
    client: Client,
    key: PrivateKey,
    pub history: Vec<History>,
}

impl ClientWithKey {
    pub fn new<K, U>(key: K, url: U) -> Result<ClientWithKey>
    where
        K: TryInto<PrivateKey>,
        <K as TryInto<PrivateKey>>::Error: std::fmt::Debug,
        U: TryInto<Url>,
        <U as TryInto<Url>>::Error: std::fmt::Debug,
    {
        let client = Client::new(url)?;
        let key = key.try_into().map_err(|err| eyre!("{err:?}"))?;

        Ok(Self::with_key_and_client(key, client))
    }

    pub fn with_key_and_client(key: PrivateKey, client: Client) -> ClientWithKey {
        Self {
            key,
            client,
            history: Default::default(),
        }
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn history<S: Display>(&mut self, query: S) -> Result<()> {
        debug!("last query: {query}");

        self.history = self.client.history(query).await?;

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn status(&self) -> Result<ApiStatus> {
        debug!("status");
        self.client.status().await
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn cost(&self, count: u64) -> Result<u64> {
        self.client.cost(count).await
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn account(&self) -> Result<AccountInfo> {
        self.client.account(&self.key.public_key()).await
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn balance(&self) -> Result<u64> {
        self.account()
            .await
            .map(|acc| acc.balance)
            .context("Error when receiving the balance")
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn airdrop(&self) -> Result<u64> {
        self.client.airdrop(&self.key.public_key()).await
    }

    pub async fn query<S: ToString>(&self, query: S) -> Result<QueryId> {
        self.client
            .send(
                "/query",
                &AiRequest::new(
                    query.to_string(),
                    self.history.clone(),
                    self.key.public_key(),
                )
                .sign(&self.key)?,
            )
            .await
    }

    pub async fn answer(&self, query_id: &QueryId) -> Result<Query> {
        self.client.answer(query_id).await
    }

    #[cfg(feature = "time")]
    #[instrument(level = "debug", skip(self))]
    pub async fn answer_wait(
        &self,
        query_id: &QueryId,
        duration: Option<Duration>,
    ) -> Result<Query> {
        self.client.answer_wait(query_id, duration).await
    }

    pub async fn get_ai(&self) -> Result<AiDownloadModel> {
        self.client.get_ai().await
    }

    pub fn public_key(&self) -> PublicKey {
        self.key.public_key()
    }
}

impl Deref for ClientWithKey {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

#[derive(Debug, Deserialize)]
pub struct AccountInfo {
    balance: u64,
}

#[derive(Debug, Deserialize)]
pub struct ApiStatus {
    cost: u64,
}
