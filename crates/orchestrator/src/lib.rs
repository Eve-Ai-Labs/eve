mod error;
mod interface;
mod network;
mod orch;
mod store;
mod tasks;
mod verifier;

use ai::Ai;
use crypto::ed25519::private::PrivateKey;
pub use error::*;
use eyre::{Context, Error, Result};
pub use interface::*;
use node_config::tasks::AiTasksConfig;
use orch::OrchestratorTask;
use std::sync::Arc;
use storage::EveStorage;
use tokio::task::JoinHandle;
use tracing::debug;
use types::p2p::Peer;
use verifier::VerifierTask;

pub async fn spawn_orchestrator<A: Ai + Send + Sync + 'static>(
    storage: Arc<EveStorage>,
    api_rec: ApiReceiver,
    p2p: (ToP2P, FromP2P),
    ai: Arc<A>,
    key: PrivateKey,
    cfg: &AiTasksConfig,
) -> Result<OrchestratorHandles, Error> {
    init_cluster(&storage, cfg)?;

    let (evaluator_rec_tx, evaluator_rec_rx) = tokio::sync::mpsc::channel(100);

    let mut evaluator = VerifierTask::new(key.clone(), ai, evaluator_rec_rx);
    let eva = tokio::spawn(async move {
        evaluator.run().await;
    });

    let mut orchestrator = OrchestratorTask::new(
        key.public_key(),
        api_rec,
        p2p,
        evaluator_rec_tx,
        storage,
        cfg,
    )?;

    let orch = tokio::spawn(async move {
        if let Err(err) = orchestrator.run().await {
            tracing::error!("Error running orchestrator: {}", err);
        }
    });

    Ok(OrchestratorHandles { ev: eva, orch })
}

fn init_cluster(storage: &EveStorage, resolver: &AiTasksConfig) -> Result<(), Error> {
    if !storage.cluster_table.is_empty()? {
        debug!("Cluster already initialized");
        return Ok(());
    }

    let mut ws = storage::WriteSet::default();

    for node in &resolver.whitelist {
        let node = Peer {
            public_key: node.public_key,
            address: node.address.clone(),
        };
        storage.cluster_table.add_node(&node, &mut ws)?;
    }
    storage.commit(ws)?;
    debug!("Cluster initialized");

    Ok(())
}

pub struct OrchestratorHandles {
    pub ev: JoinHandle<()>,
    pub orch: JoinHandle<()>,
}

impl OrchestratorHandles {
    pub async fn wait(&mut self) -> Result<()> {
        let result = tokio::select! {
            result = &mut self.ev => result.context("running evaluator"),
            result = &mut self.orch => result.context("running orchestrator")
        };

        self.abort();

        result
    }

    pub fn abort(&self) {
        self.ev.abort();
        self.orch.abort();
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use crate::{ApiReceiver, ApiSender};
    use crypto::ed25519::private::PrivateKey;
    use p2p::key::ToP2P;
    use std::{collections::HashMap, sync::Arc};
    use storage::{EveStorage, WriteSet};
    use tokio::task::JoinHandle;
    use tracing::debug;
    use types::{
        ai::query::{query_id, Query},
        cluster::{ClusterInfo, ClusterInfoWithNodes, Node},
    };

    pub struct OrchestratorMock;
    impl OrchestratorMock {
        pub fn create(storage: Arc<EveStorage>) -> (ApiSender, JoinHandle<()>) {
            let (sender, mut r): (ApiSender, ApiReceiver) = tokio::sync::mpsc::channel(100);

            let node_key = PrivateKey::generate();
            let mut nodes = HashMap::new();
            let handle = tokio::spawn(async move {
                while let Some(req) = r.recv().await {
                    match req {
                        crate::OrchRequest::Ask { request, tx } => {
                            let request = request.into_inner();
                            debug!(target = "request", "{request:?}");
                            debug!("handle user request with pubkey: {}", request.query.pubkey);

                            let result = (|| {
                                let mut ws = WriteSet::default();
                                let user_seq = storage
                                    .sequence_table
                                    .increment_and_get(&request.query.pubkey, &mut ws)?;

                                let query =
                                    Arc::new(Query::new(query_id(0, &request), user_seq, request));
                                storage.query_table.put_query(&query, &mut ws)?;
                                storage.commit(ws)?;
                                debug!("Query {} is in progress", query.id);

                                Ok(query.id)
                            })();

                            tx.send(result).unwrap();
                        }
                        crate::OrchRequest::AddNode {
                            address,
                            public_key,
                            tx,
                        } => {
                            let peer_id = public_key.to_p2p().to_peer_id();
                            nodes.insert(peer_id, Node::new(public_key, peer_id, address));
                            tx.send(Ok(())).unwrap();
                        }
                        crate::OrchRequest::RemoveNode {
                            public_key: address,
                            tx,
                        } => {
                            nodes.remove(&address.to_p2p().to_peer_id());
                            tx.send(Ok(())).unwrap();
                        }
                        crate::OrchRequest::ClusterInfo { tx } => {
                            tx.send(ClusterInfoWithNodes {
                                cluster_info: ClusterInfo {
                                    orch_address: vec![],
                                    webrtc_certhash: None,
                                    orch_pubkey: node_key.public_key(),
                                    nodes_count: nodes.len(),
                                },
                                nodes: nodes.clone(),
                            })
                            .unwrap();
                        }
                        crate::OrchRequest::Airdrop {
                            address: _,
                            amount: _,
                            tx,
                        } => {
                            tx.send(Ok(())).unwrap();
                        }
                    }
                }
            });

            (sender, handle)
        }
    }
}
