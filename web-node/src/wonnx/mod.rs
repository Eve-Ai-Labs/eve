mod task;
mod worker;

use crate::ConvertWasmResultError as _;
use ai::{error::AiError, Ai, Answer, Question};
use events::{ai::send_ai_job_status, loader::LoadStatus};
use futures::{
    channel::mpsc::{self, Sender},
    SinkExt as _, StreamExt as _,
};
use p2p::sys::now_secs;
use tracing::{info, warn};
use wasm_bindgen::JsValue;
use worker::{GenerateStatus, WonnxMessage};

pub struct Wonnx {
    tx: Sender<task::WonnxTask>,
}

impl Wonnx {
    pub fn new() -> Result<Self, JsValue> {
        Ok(Self {
            tx: task::spawn_wonnx_task()?,
        })
    }

    pub async fn load_model<P>(&self, progress: P) -> Result<(), JsValue>
    where
        P: Fn(Result<LoadStatus, JsValue>),
    {
        info!("Loading the model");
        let (tx, mut rx) = mpsc::channel(10);

        self.tx
            .clone()
            .send(task::WonnxTask::LoadModel(tx))
            .await
            .error_to_js()?;

        loop {
            let message = match rx.next().await {
                Some(Ok(message)) => message,
                Some(Err(error)) => {
                    let err =
                        JsValue::from_str(format!("Failed to load the model. {}", error).as_str());
                    progress(Err(err.clone()));
                    return Err(err);
                }
                None => {
                    let err = JsValue::from_str("Failed to load the model");
                    progress(Err(err.clone()));
                    return Err(err);
                }
            };

            match message {
                WonnxMessage::Loading(loading) => {
                    let is_done = LoadStatus::Done == loading;
                    progress(Ok(loading));
                    if is_done {
                        return Ok(());
                    }
                }
                _ => {
                    let err = JsValue::from_str("Failed to load the model. Unexpected message");
                    progress(Err(err.clone()));
                    return Err(err);
                }
            };
        }
    }

    async fn generate(&self, question: Question) -> Result<Answer, JsValue> {
        info!("Generate response");
        let (tx, mut rx) = mpsc::channel(2);

        self.tx
            .clone()
            .send(task::WonnxTask::Generate(question, tx))
            .await
            .error_to_js()?;

        loop {
            let message = match rx.next().await {
                Some(Ok(message)) => message,
                Some(Err(error)) => {
                    let err = JsValue::from_str(
                        format!("Failed to generate response. {}", error).as_str(),
                    );
                    return Err(err);
                }
                None => {
                    let err = JsValue::from_str("Failed to generate response");
                    return Err(err);
                }
            };
            match message {
                WonnxMessage::Generate(generate) => match generate {
                    GenerateStatus::Update { tps } => {
                        if let Err(err) = send_ai_job_status(&events::ai::AiJob::Update { tps }) {
                            warn!("Failed to send job status: {:?}", err);
                        }
                    }
                    GenerateStatus::Done(answer) => return Ok(answer),
                    _ => {}
                },
                _ => {
                    let err = JsValue::from_str("Failed to generate response. Unexpected message");
                    return Err(err);
                }
            };
        }
    }
}

impl Ai for Wonnx {
    async fn ask(&self, question: Question) -> Result<Answer, AiError> {
        let start = now_secs();
        let request_len = question.length();

        if let Err(err) = send_ai_job_status(&events::ai::AiJob::Started) {
            warn!("Failed to send job status: {:?}", err);
        }

        self.generate(question)
            .await
            .map_err(|err| {
                warn!("Failed to generate response. {:?}", err);
                AiError::InternalError
            })
            .inspect(|answer| {
                if let Err(err) = send_ai_job_status(&events::ai::AiJob::Done) {
                    warn!("Failed to send job status: {:?}", err);
                }

                let end = now_secs();

                crate::info!(
                    "Request processed in {}s, request length: {} answer length: {} answer cost: {}",
                    end - start,
                    request_len,
                    answer.message.len(),
                    answer.tokens + request_len as u64,
                );
            })
    }
}
