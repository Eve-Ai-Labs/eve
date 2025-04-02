use crate::results::{ConvertWasmResultError, ToJsValue, WebResult};
use crypto::ed25519::private::PrivateKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct Settings {
    pub private_key: Option<PrivateKey>,
}

impl Settings {
    pub(crate) async fn save(&self) -> WebResult {
        let win = web_sys::window().ok_or("window not found".to_js()?)?;
        let store = win
            .local_storage()?
            .ok_or("local storage not found".to_js()?)?;
        match &self.private_key {
            Some(key) => {
                store.set("private_key", &key.to_hex())?;
            }
            None => store.remove_item("private_key")?,
        }

        "The settings have been saved successfully".to_js()
    }

    pub(crate) async fn load(&mut self) -> WebResult {
        let win = web_sys::window().ok_or("window not found".to_js()?)?;
        let store = win
            .local_storage()?
            .ok_or("local storage not found".to_js()?)?;
        if let Some(private_key) = store.get("private_key")? {
            self.private_key = Some(private_key.parse().error_to_js()?);
        }

        "The settings has been loaded".to_js()
    }
}
