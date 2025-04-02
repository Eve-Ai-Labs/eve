use crate::{error::NodeError, net::Network, FromP2P, ToP2P};
use ai::{Ai, QuestionOptions};
use crypto::ed25519::{private::PrivateKey, public::PublicKey};
use futures::{SinkExt as _, StreamExt};
use multiaddr::Multiaddr;
use p2p::{etp::FromETP, sys::now_secs, task::PeerId};
use std::sync::Arc;
use tracing::{error, info, warn};
use types::{
    ai::{
        query::QueryId,
        request::SignedAiRequest,
        response::{AiResponse, SignedAiResponse},
    },
    p2p::{EveMessage, NodeMessage},
};

pub struct NodeTask<A> {
    to_p2p: ToP2P,
    from_p2p: FromP2P,
    ai: Arc<A>,
    node_key: PrivateKey,
    network: Network,
}

impl<A: Ai + Send + Sync + 'static> NodeTask<A> {
    pub fn new(
        p2p: (ToP2P, FromP2P),
        ai: Arc<A>,
        orch_public_key: PublicKey,
        node_key: PrivateKey,
        orch_address: Multiaddr,
    ) -> Self {
        let network = Network::new(orch_address, orch_public_key, p2p.0.clone());

        Self {
            to_p2p: p2p.0,
            from_p2p: p2p.1,
            ai,
            node_key,
            network,
        }
    }

    async fn handle_ai_request(
        &self,
        sender: PeerId,
        id: QueryId,
        request: SignedAiRequest,
    ) -> Result<(), NodeError> {
        if !self.network.is_orch(sender) {
            warn!("Received AI request from non-orchestrator peer {sender}");
            return Err(NodeError::InvalidSender);
        }

        let ai = self.ai.clone();
        let node_key = self.node_key.clone();
        let mut p2p = self.to_p2p.clone();

        let task = async move {
            info!("Received AI request {id} from orchestrator");
            let response = Self::request_task(request, ai, node_key)
                .await
                .map_err(|err| err.to_string());

            let result = p2p
                .send(p2p::etp::ToETP::Send {
                    to: sender,
                    message: EveMessage::Node(NodeMessage::AiResponse { id, response }),
                    on_received: None,
                })
                .await;
            info!(
                "Sent response to orchestrator for request {id}:{}",
                result.is_ok()
            );
            if let Err(err) = result {
                error!("Failed to send response: {err}");
            }
        };

        #[cfg(not(target_arch = "wasm32"))]
        tokio::spawn(task);
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
        Ok(())
    }

    async fn request_task(
        request: SignedAiRequest,
        ai: Arc<A>,
        key: PrivateKey,
    ) -> Result<SignedAiResponse, NodeError> {
        let request = request
            .verify()
            .map_err(|_| NodeError::InvalidSignature)?
            .into_inner();

        let request_signature = request.signature().to_owned();
        let question = ai::Question {
            message: request.query.message,
            history: request.query.history,
            options: QuestionOptions {
                seed: request.query.seed,
                ..Default::default()
            },
        };
        let answer: ai::Answer = ai.ask(question).await?;

        AiResponse {
            response: answer.message,
            pubkey: key.public_key(),
            request_signature,
            timestamp: now_secs(),
            cost: answer.tokens,
        }
        .sign(&key)
        .map_err(NodeError::FailedToSignResponse)
    }

    async fn handle_message(&mut self, msg: FromETP<EveMessage>) -> Result<(), NodeError> {
        match msg {
            FromETP::Receive(peer_id, msg) => match msg {
                EveMessage::Orch(orch_message) => match orch_message {
                    types::p2p::OrchMessage::AiRequest { id, request } => {
                        self.handle_ai_request(peer_id, id, request).await?;
                    }
                },
                EveMessage::Node(_) => {
                    warn!("Received node message from node {peer_id}");
                }
            },
            FromETP::Connect(peer_id) => self.network.connect_peer(peer_id).await?,
            FromETP::Disconnect(peer_id) => self.network.disconnect_peer(peer_id).await?,
        }
        Ok(())
    }

    async fn handle_request(
        &mut self,
        request: Option<FromETP<EveMessage>>,
    ) -> Result<(), NodeError> {
        if let Some(request) = request {
            let result = self.handle_message(request).await;
            if matches!(&result, Err(NodeError::P2PError)) {
                warn!("P2P error. Exiting node task");
                return Err(NodeError::P2PError);
            }
            if let Err(err) = result {
                warn!("Error handling message: {err:#?}");
            }
        } else {
            warn!("P2P receiver closed");
            return Err(NodeError::P2PError);
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), NodeError> {
        info!("Starting node task");
        self.network.reconnect_nodes().await?;

        info!("Dialing complete");
        #[cfg(not(target_arch = "wasm32"))]
        {
            use futures::FutureExt;
            let mut reconnect_interval = tokio::time::interval(std::time::Duration::from_secs(5));
            loop {
                futures::select! {
                    request = self.from_p2p.next() => {
                        self.handle_request(request).await?
                    }
                    _ = reconnect_interval.tick().fuse() => {
                       self.network.reconnect_nodes().await?
                    }
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            loop {
                futures::select! {
                    request = self.from_p2p.next() => {
                        self.handle_request(request).await?;
                    }
                }
            }
        }
    }
}
