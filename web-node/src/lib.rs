pub mod client;
mod env;
pub(crate) mod log;
pub(crate) mod results;
pub mod settings;
mod wonnx;

use crate::{
    results::{ConvertWasmResult, ConvertWasmResultError, ToJsValue, WebResult},
    settings::EveSettings,
};
use crypto::ed25519::public::PublicKey;
use env::get_whitelist_form;
use events::loader::send_load_status;
use futures::SinkExt;
use node::{spawn_node, ToP2P};
use orchestrator_client::ClientWithKey;
use p2p::{spawn, sys::sleep, Config};
use std::sync::Arc;
use tracing::Level;
use tracing_wasm::WASMLayerConfigBuilder;
use types::p2p::EveMessage;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::Storage;
use wonnx::Wonnx;

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn main() {
    tracing_wasm::set_as_global_default_with_config(
        WASMLayerConfigBuilder::new()
            .set_max_level(Level::INFO)
            .build(),
    );
}

#[wasm_bindgen]
pub fn get_api() -> String {
    env::get_api_orchestrator().to_string()
}

#[wasm_bindgen]
pub struct EveNode {
    settings: EveSettings,
    client: Option<ClientWithKey>,
    node: Option<Node>,
    wonnx: Arc<Wonnx>,
}

#[wasm_bindgen]
impl EveNode {
    pub async fn new() -> Result<Self, JsValue> {
        let wonnx = Arc::new(Wonnx::new()?);

        let settings = EveSettings::load().await?;
        let mut web_node = Self {
            settings,
            client: None,
            node: None,
            wonnx,
        };

        // client
        web_node
            .recreate_client()
            .inspect_err(|err| debug!("{err:#?}"))?;

        Ok(web_node)
    }

    pub async fn start(&mut self) -> WebResult {
        if self.is_running() {
            return Err("The node is already running").error_to_js();
        }

        self.recreate_client()
            .inspect_err(|err| debug!("{err:?}"))?;
        tracing::info!("Starting the node");

        let client = self
            .client
            .as_ref()
            .ok_or("The client is not configured. Call set_settings before starting the node")
            .error_to_js()?;

        let info = client.cluster_info().await.error_to_js()?;
        tracing::info!("Cluster info: {:#?}", info);
        let private_key = self.settings.private_key.clone().ok_or({
            "The private key is not configured. Call set_settings before starting the node"
        })?;

        let node_key = private_key.public_key();

        self.wait_connection(node_key).await?;

        let webrtc = if let Some(webrtc) = info.find_webrtc().error_to_js()? {
            tracing::info!("Webrtc certhash: {}", webrtc);
            webrtc
        } else {
            return Err("webrtc certhash not found").error_to_js();
        };

        self.load_model().await?;

        tracing::info!("Connecting to the orchestrator: {}", webrtc);

        let (_, rx, tx) = spawn::<EveMessage>(
            private_key,
            info.orch_pubkey,
            vec![webrtc.clone()],
            &[],
            Config::default(),
        )
        .await
        .map_err(|err| format!("Error spawning p2p: {}", err))?;

        spawn_node(
            tx.clone(),
            rx,
            self.wonnx.clone(),
            info.orch_pubkey,
            self.settings.private_key.clone().unwrap(),
            webrtc,
        )
        .await
        .map_err(|err| format!("Error spawning node: {}", err))?;

        self.node = Some(Node { p2p: tx });
        info!("Node started");
        "".to_js()
    }

    pub async fn stop(&mut self) -> WebResult {
        tracing::info!("Stopping the node");
        self.wonnx = Arc::new(Wonnx::new()?);

        if let Some(mut node) = self.node.take() {
            node.p2p.send(p2p::etp::ToETP::Shutdown).await.to_js()?;
            info!("The node is stopped");
        } else {
            tracing::info!("The node is not running");
        }
        "".to_js()
    }

    pub async fn stop_wait_disconnect(&mut self) -> WebResult {
        self.stop().await?;

        let (Some(key), Some(client)) = (self.settings.private_key.as_ref(), self.client.as_ref())
        else {
            return "".to_js();
        };
        let account = key.public_key();

        let mut limit = 180;
        loop {
            let node_opt = client.node(account).await.error_to_js()?;
            tracing::info!("Node: {:?}", node_opt);
            if let Some(node) = node_opt {
                if !node.is_connected {
                    return "Stopped".to_js();
                }
            }

            limit -= 1;
            if limit == 0 {
                return Err("Timeout").error_to_js();
            }
            sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    pub fn is_running(&self) -> bool {
        self.node.is_some()
    }

    fn recreate_client(&mut self) -> Result<(), JsValue> {
        self.client = self.settings.client()?;
        Ok(())
    }

    async fn load_model(&self) -> Result<(), JsValue> {
        tracing::info!("Loading the model");

        self.wonnx
            .load_model(|p| {
                if let Err(err) = send_load_status(&p.map_err(|err| format!("{:?}", err))) {
                    tracing::error!("Failed to send load status: {:?}", err);
                }
            })
            .await?;
        tracing::info!("Model loaded");
        Ok(())
    }

    async fn wait_connection(&self, key: PublicKey) -> Result<(), JsValue> {
        tracing::info!("Waiting for the connection");
        let mut try_count = 10;
        let client = self.client.as_ref().ok_or("The client is not configured")?;
        loop {
            let node_opt = client.node(key).await.error_to_js()?;
            tracing::info!("Node: {:?}", node_opt);
            if let Some(node) = node_opt {
                if !node.is_connected {
                    return Ok(());
                }
            } else if let Some(form) = get_whitelist_form() {
                return Err(format!(
                    "Please add the node {key}\n to the whitelist first <a href=\"{form}?key={key}\" target=\"_blank\">{form}</a>",
                ))
                .error_to_js();
            } else {
                return Err(format!(
                    "Please add the node {key}\n to the whitelist first.",
                ))
                .error_to_js();
            }
            sleep(std::time::Duration::from_secs(1)).await;

            try_count -= 1;
            if try_count == 0 {
                return Err(format!(
                    "No is already connected to the orchestrator: {}",
                    key
                ))
                .error_to_js();
            }
        }
    }
}

struct Node {
    p2p: ToP2P,
}

impl Drop for Node {
    fn drop(&mut self) {
        self.p2p.close_channel();
    }
}

pub(crate) async fn storage() -> Result<Storage, JsValue> {
    let win = web_sys::window().ok_or("window not found".to_js()?)?;
    win.local_storage()?
        .ok_or("local storage not found".to_js()?)
}
