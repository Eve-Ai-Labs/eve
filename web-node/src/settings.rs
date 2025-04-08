use crate::{
    env::get_api_orchestrator,
    results::{ConvertWasmResultError, ToJsValue, WebResult},
    storage,
};
use crypto::ed25519::private::PrivateKey;
use eyre::Result;
use orchestrator_client::ClientWithKey;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EveSettings {
    pub(crate) private_key: Option<PrivateKey>,
}

#[wasm_bindgen]
impl EveSettings {
    pub async fn load() -> Result<EveSettings, JsValue> {
        let private_key = storage()
            .await?
            .get("private_key")?
            .map(|private_key| private_key.parse().error_to_js())
            .transpose()?;
        Ok(EveSettings { private_key })
    }

    pub async fn get(&self) -> WebResult {
        self.to_js()
    }

    pub async fn save(&self) -> WebResult {
        let store = storage().await?;
        match &self.private_key {
            Some(key) => {
                store.set("private_key", &key.to_hex())?;
            }
            None => store.remove_item("private_key")?,
        }

        "The settings have been saved successfully".to_js()
    }

    pub async fn set_private_key(&mut self, settings: JsValue) -> WebResult {
        let private_key = serde_wasm_bindgen::from_value(settings).error_to_js()?;
        self.private_key = Some(private_key);
        self.save().await
    }

    pub(crate) fn client(&self) -> Result<Option<ClientWithKey>, JsValue> {
        self.private_key
            .as_ref()
            .map(|key| ClientWithKey::new(key.clone(), get_api_orchestrator()).error_to_js())
            .transpose()
    }
}
