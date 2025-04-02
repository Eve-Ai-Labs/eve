pub mod error;
mod net;
mod task;

use ai::Ai;
use crypto::ed25519::{private::PrivateKey, public::PublicKey};
use eyre::Error;
use multiaddr::Multiaddr;
use p2p::etp::{FromETP, ToETP};
use std::sync::Arc;
use tracing::error;
use types::p2p::EveMessage;

pub type ToP2P = futures::channel::mpsc::Sender<ToETP<EveMessage>>;
pub type FromP2P = futures::channel::mpsc::Receiver<FromETP<EveMessage>>;

pub async fn spawn_node<A: Ai + Send + Sync + 'static>(
    to_p2p: ToP2P,
    from_p2p: FromP2P,
    ai: Arc<A>,
    orch_public_key: PublicKey,
    private_key: PrivateKey,
    orch_address: Multiaddr,
) -> Result<NodeHandler, Error> {
    let mut task = task::NodeTask::new(
        (to_p2p, from_p2p),
        ai,
        orch_public_key,
        private_key,
        orch_address,
    );

    let task = async move {
        let result = task.run().await;
        if let Err(e) = result {
            error!("Node task failed: {}", e);
        }
    };
    #[cfg(not(target_arch = "wasm32"))]
    let handler = tokio::spawn(task);
    #[cfg(target_arch = "wasm32")]
    let handler = wasm_bindgen_futures::spawn_local(task);

    Ok(NodeHandler { handler })
}

pub struct NodeHandler {
    #[cfg(not(target_arch = "wasm32"))]
    pub handler: tokio::task::JoinHandle<()>,
    #[cfg(target_arch = "wasm32")]
    pub handler: (),
}
