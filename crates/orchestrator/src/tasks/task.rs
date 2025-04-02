use super::{env::Env, NodeResponse};
use crate::{
    network::ConnectedNode,
    verifier::{VerificationRequest, VerificationResponse},
    OrchestratorError,
};
use crypto::ed25519::public::PublicKey;
use metrics::{LATENCY, TIMEOUTS};
use rand::Rng as _;
use std::{sync::Arc, time::Duration};
use tokio::{
    select,
    sync::{mpsc::Receiver, oneshot},
    time::{sleep_until, Instant},
};
use tracing::{info, warn};
use types::ai::query::{NodeResult, Query, QueryId};

pub struct Task {
    query: Option<Query>,
    peer_pool: Vec<ConnectedNode>,
    env: Arc<Env>,
    rx: Receiver<NodeResponse>,
    used_nodes: Vec<ConnectedNode>,
    verifier_results: Vec<oneshot::Receiver<VerificationResponse>>,
}

impl Task {
    pub fn new(
        query: Query,
        peer_pool: Vec<ConnectedNode>,
        env: Arc<Env>,
        rx: Receiver<NodeResponse>,
    ) -> Self {
        Self {
            query: Some(query),
            peer_pool,
            env,
            rx,
            used_nodes: vec![],
            verifier_results: vec![],
        }
    }

    pub fn id(&self) -> &QueryId {
        &self.query.as_ref().expect("Query is not set").id
    }

    fn response_nodes(&self) -> usize {
        self.query
            .as_ref()
            .expect("Query is not set")
            .response
            .len()
    }

    pub async fn run(
        &mut self,
        tx: oneshot::Sender<Result<QueryId, OrchestratorError>>,
    ) -> Result<(), OrchestratorError> {
        info!("Spawn task for query: {}", self.id());
        let deadline = Instant::now() + Duration::from_secs(self.env.cfg.task_timeout_secs);

        while self.response_nodes() < self.env.cfg.replication_factor as usize
            && !self.peer_pool.is_empty()
            && Instant::now() < deadline
        {
            let nodes_count = self.env.cfg.replication_factor as usize - self.response_nodes();
            let nodes = self.select_workers(nodes_count).await?;
            self.query
                .as_mut()
                .expect("Query is not set")
                .response
                .extend(nodes.iter().map(|key| NodeResult::SentRequest(*key)));
        }

        let result = self.store_query().await;
        tx.send(Ok(*self.id()))
            .map_err(|_| OrchestratorError::EyreError(eyre::eyre!("Failed to send result")))?;
        result?;

        let req_ts = {
            let query = self.query.as_ref().expect("Query is not set");
            query.request.query.timestamp
        };

        if self.response_nodes() == 0 {
            warn!("Failed to send tasks to nodes. 0 Nodes responded.");
            return Ok(());
        } else {
            info!("Tasks successfully sent to {} nodes", self.response_nodes());
        }

        info!("Waiting for results for query: {}", self.id());
        loop {
            select! {
                _ = sleep_until(deadline) => {
                    TIMEOUTS.add(1, &[]);
                    self.set_timeout_error().await?;
                    break;
                }
                Some(result) = self.rx.recv() => {
                    self.set_node_result(result).await?;
                    if self.all_request_received() {
                        break;
                    }
                }
               else => {
                    break;
                }
            }
        }
        info!("Query: {}. Sending results to verifier", self.id());

        while !self.verifier_results.is_empty() {
            select! {
                _ = sleep_until(deadline) => {
                    TIMEOUTS.add(1, &[]);
                    self.set_timeout_error().await?;
                    break;
                }
                result = self.verifier_results.remove(0) => {
                    match result {
                        Ok(response) => {
                           self.set_verifier_result(response).await?;
                        }
                        Err(_) => warn!("Verifier channel closed"),
                    }
                }
            }
        }
        let res_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let latency = res_ts - req_ts;
        LATENCY.record(latency, &[]);
        info!("Query: {} completed", self.id());

        Ok(())
    }

    async fn set_verifier_result(
        &mut self,
        response: VerificationResponse,
    ) -> Result<(), OrchestratorError> {
        let query = self.query.as_mut().expect("Query is not set");
        let node_result = query
            .response
            .iter_mut()
            .find(|node| node.node_key() == response.node_key)
            .ok_or_else(|| OrchestratorError::InvalidSender)?;

        *node_result = NodeResult::Verified(Box::new(response.verification_result));

        self.store_query().await?;
        Ok(())
    }

    fn all_request_received(&self) -> bool {
        self.query
            .as_ref()
            .expect("Query is not set")
            .response
            .iter()
            .all(|node| matches!(node, NodeResult::NodeResponse(_) | NodeResult::Error(_, _)))
    }

    async fn set_node_result(&mut self, result: NodeResponse) -> Result<(), OrchestratorError> {
        let query = self.query.as_mut().expect("Query is not set");
        let node = self.used_nodes.iter().find(|node| node.peer_id == result.0);

        let send_to_verifier = {
            let node_result = if let Some(node) = node {
                let response = query.response.iter_mut().find(|n| n.node_key() == node.key);
                if let Some(response) = response {
                    response
                } else {
                    return Err(OrchestratorError::InvalidSender);
                }
            } else {
                return Err(OrchestratorError::InvalidSender);
            };

            if node_result.is_sent_request() {
                match result.1 {
                    Ok(ok) => {
                        self.env.transfer(
                            query.request.query.pubkey,
                            node_result.node_key(),
                            ok.node_response.cost,
                        )?;
                        *node_result = NodeResult::NodeResponse(ok);
                        Some(node_result.node_key())
                    }
                    Err(err) => {
                        *node_result = NodeResult::Error(node_result.node_key(), err);
                        None
                    }
                }
            } else {
                warn!("Node already responded: {:?} {:?}", query.id, node_result);
                None
            }
        };

        if let Some(node_key) = send_to_verifier {
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.env
                .send_to_evaluator(VerificationRequest::new(query, node_key, tx)?)
                .await?;
            self.verifier_results.push(rx);
        }
        self.store_query().await?;

        Ok(())
    }

    async fn set_timeout_error(&mut self) -> Result<(), OrchestratorError> {
        info!("Set timeout error for query: {}", self.id());
        let query = self.query.as_mut().expect("Query is not set");
        for node in query.response.iter_mut() {
            match node {
                NodeResult::SentRequest(public_key) => {
                    *node = NodeResult::Timeout(Box::new(NodeResult::SentRequest(*public_key)));
                }
                NodeResult::Verified(_)
                | NodeResult::Error(_, _)
                | NodeResult::NodeResponse(_)
                | NodeResult::Timeout(_) => {
                    //no-op
                }
            };
        }

        self.store_query().await
    }

    async fn store_query(&mut self) -> Result<(), OrchestratorError> {
        let env = self.env.clone();
        let query = self.query.take().expect("Query is not set");
        let query = tokio::task::spawn_blocking(move || {
            if let Err(err) = env.update_query(&query) {
                warn!("Failed to store query: {:?}", err);
            }
            query
        })
        .await?;
        self.query = Some(query);
        Ok(())
    }

    async fn select_workers(
        &mut self,
        nodes_count: usize,
    ) -> Result<Vec<PublicKey>, OrchestratorError> {
        let mut results = vec![];

        for _ in 0..nodes_count {
            if self.peer_pool.is_empty() {
                break;
            }

            let node = self
                .peer_pool
                .remove(rand::thread_rng().gen_range(0..self.peer_pool.len()));
            let result_rx = self
                .env
                .send_request(
                    node.peer_id,
                    *self.id(),
                    self.query
                        .as_ref()
                        .expect("Query is not set")
                        .request
                        .clone(),
                )
                .await;
            results.push((result_rx, node));
        }

        let mut nodes = vec![];
        for (result_rx, node) in results {
            if let Ok(rx) = result_rx {
                if let Ok(result) = rx.await {
                    if result.is_success() {
                        nodes.push(node.key);
                        self.used_nodes.push(node);
                    } else {
                        warn!(
                            "Failed to process request on node: {:?} {:?}",
                            node.key, result
                        );
                    }
                } else {
                    warn!("Failed to send request to node: {:?}", node.key);
                }
            }
        }

        Ok(nodes)
    }
}
