use crate::{
    results::WebResult, settings::EveSettings, storage, ConvertWasmResult, ConvertWasmResultError,
    ToJsValue,
};
use orchestrator_client::ClientWithKey;
use types::ai::query::QueryId;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

const LAST_QUERY_INDEX: &str = "last_query";

#[wasm_bindgen]
pub struct EveClient {
    client: ClientWithKey,
    last_query: Option<QueryId>,
}

#[wasm_bindgen]
impl EveClient {
    pub async fn new() -> Result<Self, JsValue> {
        let client = EveSettings::load()
            .await?
            .client()?
            .ok_or_else(|| JsValue::from_str("The settings are not set"))?;
        Ok(EveClient {
            client,
            last_query: last_query().await?,
        })
    }

    pub async fn balance(&self) -> WebResult {
        self.client.balance().await.to_js()
    }

    pub async fn get_history(&mut self) -> WebResult {
        let Some(last_query) = self.last_query else {
            return Ok(JsValue::null());
        };
        if self.client.history.is_empty() {
            self.client.history(last_query).await.error_to_js()?;
        }

        self.client.history.to_js()
    }

    pub async fn clear_history(&mut self) -> WebResult {
        self.last_query = None;
        self.client.history = Default::default();
        clear_history().await?;

        "success".to_js()
    }

    pub async fn ask(&mut self, prompt: String) -> WebResult {
        if let Some(back_query) = self.last_query {
            self.client.history(back_query).await.error_to_js()?;
        }
        let query_id = self.client.query(&prompt).await.error_to_js()?;
        self.last_query = Some(query_id);
        save_last_query(query_id).await?;
        query_id.to_hex().to_js()
    }

    pub async fn status(&self, query_id: String) -> WebResult {
        self.client
            .answer(&query_id.parse().error_to_js()?)
            .await
            .to_js()
    }
}

async fn last_query() -> Result<Option<QueryId>, JsValue> {
    storage()
        .await?
        .get(LAST_QUERY_INDEX)?
        .map(|query_id| query_id.parse().error_to_js())
        .transpose()
}

async fn save_last_query(last_query: QueryId) -> Result<(), JsValue> {
    storage().await?.set(LAST_QUERY_INDEX, &last_query.to_hex())
}

async fn clear_history() -> Result<(), JsValue> {
    storage().await?.delete(LAST_QUERY_INDEX)
}
