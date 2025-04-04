use crate::env::url_suffix;
use ai::Answer;
use events::loader::{LoadStatus, Progress};
use eyre::Error;
use futures::{channel::mpsc::Sender, SinkExt};
use js_sys::{Array, Object, Reflect};
use tracing::warn;
use wasm_bindgen::{prelude::Closure, JsCast as _, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{MessageEvent, Worker};

const SCRIPT_PATH: &str = "/scripts/wonnx/worker.js";

type Handlers = (
    Closure<dyn FnMut(MessageEvent)>,
    Closure<dyn FnMut(MessageEvent)>,
);

pub struct WonnxWorker {
    worker: Worker,
    handlers: Option<Handlers>,
}

impl WonnxWorker {
    pub fn new() -> Result<Self, JsValue> {
        let options = web_sys::WorkerOptions::new();
        options.set_type(web_sys::WorkerType::Module);
        let script_path = SCRIPT_PATH.to_string() + &url_suffix();
        let worker = Worker::new_with_options(&script_path, &options)?;

        Ok(Self {
            worker,
            handlers: None,
        })
    }

    pub fn set_message_handler(&mut self, tx: Sender<WonnxResult>) {
        let tx_clone = tx.clone();
        let msg_handler = Closure::wrap(Box::new(move |event: MessageEvent| {
            let msg = WonnxMessage::try_from(event);
            let mut tx = tx_clone.clone();
            spawn_local(async move {
                if tx.send(msg).await.is_err() {
                    warn!("Failed to send message to the channel");
                }
            });
        }) as Box<dyn FnMut(MessageEvent)>);
        self.worker
            .set_onmessage(Some(msg_handler.as_ref().unchecked_ref()));

        let error_handler = Closure::wrap(Box::new(move |event: MessageEvent| {
            let error = eyre::eyre!("{:?}", event.data());
            let mut tx = tx.clone();
            spawn_local(async move {
                if tx.send(Err(error)).await.is_err() {
                    warn!("Failed to send message to the channel");
                }
            });
        }) as Box<dyn FnMut(MessageEvent)>);
        self.worker
            .set_onmessageerror(Some(error_handler.as_ref().unchecked_ref()));

        self.handlers = Some((msg_handler, error_handler));
    }

    fn post_message(&self, message_type: &str, data: Option<&JsValue>) -> Result<(), Error> {
        let message = Object::new();
        Reflect::set(
            &message,
            &JsValue::from_str("type"),
            &JsValue::from_str(message_type),
        )
        .map_err(|e| eyre::eyre!("Failed to set message type: {:?}", e))?;

        if let Some(data_value) = data {
            Reflect::set(&message, &JsValue::from_str("data"), data_value)
                .map_err(|e| eyre::eyre!("Failed to set message type: {:?}", e))?;
        }

        self.worker
            .post_message(&message)
            .map_err(|e| eyre::eyre!("Failed to post message type: {:?}", e))?;
        Ok(())
    }

    pub(crate) fn post_load_model(&self) -> Result<(), Error> {
        self.post_message("load", None)
    }

    pub(crate) fn post_generate(&self, question: ai::Question) -> Result<(), Error> {
        let js_messages = Array::new();

        for row in question.history {
            let obj = Object::new();
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("role"),
                &JsValue::from_str(&row.role.to_string()),
            );
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("content"),
                &JsValue::from_str(&row.content),
            );
            js_messages.push(&obj);
        }
        self.post_message("generate", Some(&JsValue::from(js_messages)))
    }
}

pub type WonnxResult = Result<WonnxMessage, Error>;

impl Drop for WonnxWorker {
    fn drop(&mut self) {
        self.worker.terminate();
    }
}

#[derive(Debug)]
pub enum WonnxMessage {
    Loading(LoadStatus),
    Generate(GenerateStatus),
}

#[derive(Debug)]
pub enum GenerateStatus {
    Start,
    Update { tps: f64 },
    Done(Answer),
}

impl TryFrom<MessageEvent> for WonnxMessage {
    type Error = Error;

    fn try_from(value: MessageEvent) -> Result<Self, Self::Error> {
        let data = value.data();
        let status = get_str_value(&data, "tp")?;

        match status.as_str() {
            "wonnx.loading" => {
                let tp = get_str_value(&data, "progress_type")?;
                let status = get_str_value(&data, "status").ok();
                match (status.as_deref(), tp.as_str()) {
                    (Some("download"), "wonnx.progress") => {
                        Ok(WonnxMessage::Loading(LoadStatus::Download(Progress {
                            progress: get_f64_value(&data, "progress").unwrap_or(0.0),
                        })))
                    }
                    (Some("progress"), "wonnx.progress") => {
                        Ok(WonnxMessage::Loading(LoadStatus::Progress(Progress {
                            progress: get_f64_value(&data, "progress").unwrap_or(0.0),
                        })))
                    }
                    (_, "wonnx.started")
                    | (Some("initiate"), _)
                    | (Some("donwload"), _)
                    | (Some("done"), _) => Ok(WonnxMessage::Loading(LoadStatus::Start)),
                    (_, "wonnx.compile") => Ok(WonnxMessage::Loading(LoadStatus::Compile)),
                    (_, "wonnx.complete") => Ok(WonnxMessage::Loading(LoadStatus::Done)),
                    _ => Err(eyre::eyre!("Unknown progress_type: {status:?} {tp}")),
                }
            }
            "wonnx.generate" => {
                let tp = get_str_value(&data, "progress_type")?;
                match tp.as_str() {
                    "wonnx.started" => Ok(WonnxMessage::Generate(GenerateStatus::Start)),
                    "wonnx.progress" => {
                        let tps = get_f64_value(&data, "tps").unwrap_or(0.0);
                        Ok(WonnxMessage::Generate(GenerateStatus::Update { tps }))
                    }
                    "wonnx.complete" => {
                        let mut output = get_str_array_value(&data, "output")?;
                        if output.is_empty() {
                            return Err(eyre::eyre!("Empty output"));
                        }
                        let tokens = get_f64_value(&data, "numTokens").unwrap_or(0.0) as u64;
                        Ok(WonnxMessage::Generate(GenerateStatus::Done(Answer {
                            message: output.remove(0),
                            tokens,
                        })))
                    }
                    _ => Err(eyre::eyre!("Unknown progress_type: {}", tp)),
                }
            }
            _ => Err(eyre::eyre!("Unknown status: {}", status)),
        }
    }
}

fn get_str_array_value(data: &JsValue, key: &str) -> Result<Vec<String>, Error> {
    let value = Reflect::get(data, &JsValue::from_str(key))
        .map_err(|_| eyre::eyre!("Failed to get {}", key))?;

    let js_array = value
        .dyn_into::<Array>()
        .map_err(|_| eyre::eyre!("{} is not an array", key))?;

    let mut result = Vec::with_capacity(js_array.length() as usize);
    for i in 0..js_array.length() {
        if let Some(item) = js_array.get(i).as_string() {
            result.push(item);
        }
    }
    Ok(result)
}

fn get_str_value(data: &JsValue, key: &str) -> Result<String, Error> {
    Reflect::get(data, &JsValue::from_str(key))
        .map_err(|_| eyre::eyre!("Failed to get {}. Data:{:?}", key, data))
        .and_then(|v| {
            v.as_string()
                .ok_or_else(|| eyre::eyre!("{} is not a string. Data:{:?}", key, data))
        })
}

fn get_f64_value(data: &JsValue, key: &str) -> Result<f64, Error> {
    Reflect::get(data, &JsValue::from_str(key))
        .map_err(|_| eyre::eyre!("Failed to get {}. Data:{:?}", key, data))
        .and_then(|v| {
            v.as_f64()
                .ok_or_else(|| eyre::eyre!("{} is not a f64. Data:{:?}", key, data))
        })
}
