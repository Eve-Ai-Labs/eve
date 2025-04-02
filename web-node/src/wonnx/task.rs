use super::worker::{self, WonnxResult};
use ai::Question;
use futures::{
    channel::mpsc::{self, Sender},
    SinkExt as _, StreamExt as _,
};
use tracing::{info, warn};
use wasm_bindgen::JsValue;

pub enum WonnxTask {
    LoadModel(Sender<WonnxResult>),
    Generate(Question, Sender<WonnxResult>),
}

pub fn spawn_wonnx_task() -> Result<Sender<WonnxTask>, JsValue> {
    let mut worker = worker::WonnxWorker::new()?;
    let (tx, mut rx) = mpsc::channel::<WonnxTask>(10);

    wasm_bindgen_futures::spawn_local(async move {
        while let Some(task) = rx.next().await {
            match task {
                WonnxTask::LoadModel(tx) => {
                    info!("Loading the model");
                    load_model(&mut worker, tx).await;
                }
                WonnxTask::Generate(question, tx) => {
                    info!("Generate response");
                    generate(&mut worker, question, tx).await;
                }
            }
        }
    });
    Ok(tx)
}

async fn load_model(worker: &mut worker::WonnxWorker, mut tx: Sender<WonnxResult>) {
    worker.set_message_handler(tx.clone());
    if let Err(err) = worker.post_load_model() {
        if tx.send(Err(err)).await.is_err() {
            warn!("Failed to send message to the channel");
        }
    }
}

async fn generate(
    worker: &mut worker::WonnxWorker,
    question: Question,
    mut tx: Sender<WonnxResult>,
) {
    worker.set_message_handler(tx.clone());
    if let Err(err) = worker.post_generate(question) {
        if tx.send(Err(err)).await.is_err() {
            warn!("Failed to send message to the channel");
        }
    }
}
