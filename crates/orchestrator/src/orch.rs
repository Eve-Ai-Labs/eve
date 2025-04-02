use crate::{
    error::OrchestratorError,
    network::Network,
    store::{accounts::Accounts, queries::Queries},
    tasks::Tasks,
    verifier::VerificationRequest,
    ApiReceiver, FromP2P, OrchRequest, ToP2P,
};
use crypto::ed25519::public::PublicKey;
use eyre::Error;
use futures::StreamExt;
use metrics::ERRORS;
use node_config::tasks::AiTasksConfig;
use p2p::etp::FromETP;
use std::sync::Arc;
use storage::EveStorage;
use tokio::sync::mpsc::Sender;
use tracing::warn;
use types::p2p::{EveMessage, NodeMessage};

pub(crate) struct OrchestratorTask {
    api_receiver: ApiReceiver,
    p2p_receiver: FromP2P,
    tasks: Tasks,
    net: Network,
    accounts: Accounts,
}

impl OrchestratorTask {
    pub fn new(
        key: PublicKey,
        api_receiver: ApiReceiver,
        p2p: (ToP2P, FromP2P),
        verifier: Sender<VerificationRequest>,
        store: Arc<EveStorage>,
        cfg: &AiTasksConfig,
    ) -> Result<Self, Error> {
        let accounts = Accounts::new(store.clone());
        let queries = Queries::new(store.clone());
        let net = Network::new(key, p2p.0.clone(), store)?;

        Ok(Self {
            api_receiver,
            p2p_receiver: p2p.1,
            net,
            tasks: Tasks::new(queries, accounts.clone(), cfg, verifier, p2p.0),
            accounts,
        })
    }

    async fn handle_nodes_request(
        &mut self,
        msg: FromETP<EveMessage>,
    ) -> Result<(), OrchestratorError> {
        match msg {
            FromETP::Receive(sender, msg) => match msg {
                EveMessage::Orch(orch_message) => {
                    warn!("Received Orch message: {:?}", orch_message);
                    Ok(())
                }
                EveMessage::Node(NodeMessage::AiResponse { id, response }) => {
                    self.tasks.on_node_response(id, sender, response).await;
                    Ok(())
                }
            },
            FromETP::Connect(peer_id) => self.net.connect_peer(peer_id),
            FromETP::Disconnect(peer_id) => self.net.disconnect_peer(peer_id),
        }
    }

    async fn handle_api_request(&mut self, request: OrchRequest) -> Result<(), OrchestratorError> {
        match request {
            OrchRequest::Ask { request, tx } => {
                let result = self.tasks.new_task(request, &self.net, tx).await;
                if matches!(&result, Err(OrchestratorError::P2PError)) {
                    warn!("Failed to send message to p2p");
                    return Err(OrchestratorError::P2PError);
                }
            }
            OrchRequest::AddNode {
                address,
                public_key,
                tx,
            } => {
                let result = self.net.add_to_cluster(public_key, address).await;
                if matches!(&result, Err(OrchestratorError::P2PError)) {
                    warn!("Failed to send message to p2p");
                    if let Err(e) = tx.send(result) {
                        warn!("Failed to send query id to api: {:?}", e);
                    }
                    return Err(OrchestratorError::P2PError);
                }
                if let Err(e) = tx.send(result) {
                    warn!("Failed to send query id to api: {:?}", e);
                }
            }
            OrchRequest::RemoveNode {
                public_key: address,
                tx,
            } => {
                let result = self.net.remove_from_cluster(address).await;
                if matches!(&result, Err(OrchestratorError::P2PError)) {
                    warn!("Failed to send message to p2p");
                    if let Err(e) = tx.send(result) {
                        warn!("Failed to send query id to api: {:?}", e);
                    }
                    return Err(OrchestratorError::P2PError);
                }
                if let Err(e) = tx.send(result) {
                    warn!("Failed to send query id to api: {:?}", e);
                }
            }
            OrchRequest::ClusterInfo { tx } => {
                let info = self.net.cluster_info().await?;
                if let Err(e) = tx.send(info) {
                    warn!("Failed to send query id to api: {:?}", e);
                }
            }
            OrchRequest::Airdrop {
                address,
                amount,
                tx,
            } => {
                let accounts = self.accounts.clone();
                tokio::task::spawn_blocking(move || {
                    if let Err(e) = tx.send(accounts.airdrop(address, amount)) {
                        warn!("Failed to send query id to api: {:?}", e);
                    }
                });
            }
        }

        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        self.net.init_whitelist().await?;
        let mut expire_task_interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

        loop {
            tokio::select! {
                Some(request) = self.api_receiver.recv() => {
                    let result = self.handle_api_request(request).await;
                    if matches!(&result, Err(OrchestratorError::P2PError)) {
                       warn!("Failed to send message to p2p");
                       ERRORS.add(1, &[]);
                       return Err(eyre::eyre!("Failed to send message to p2p"));
                    }
                    if let Err(err) = result {
                        ERRORS.add(1, &[]);
                        warn!("Failed to handle api request: {:?}", err);
                    }
                }
                Some(request) = self.p2p_receiver.next() => {
                    let result = self.handle_nodes_request(request).await;
                    if matches!(&result, Err(OrchestratorError::P2PError)) {
                       warn!("Failed to send message to p2p");
                       ERRORS.add(1, &[]);
                       return Err(eyre::eyre!("Failed to send message to p2p"));
                    }
                    if let Err(e) = result {
                        ERRORS.add(1, &[]);
                        warn!("Failed to handle p2p request: {:?}", e);
                    }
                }
                _ = expire_task_interval.tick() => {
                    self.tasks.gc_tasks();
                }
            }
        }
    }
}
