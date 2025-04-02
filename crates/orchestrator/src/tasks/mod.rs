mod env;
mod task;

use crate::{
    network::Network,
    store::{accounts::Accounts, queries::Queries},
    verifier::VerificationRequest,
    OrchestratorError, ToP2P,
};
use env::Env;
use metrics::{ERRORS, PROCESSING, REQUESTS};
use multiaddr::PeerId;
use node_config::tasks::AiTasksConfig;
use std::{collections::HashMap, sync::Arc};
use task::Task;
use tokio::sync::{mpsc, mpsc::Sender, oneshot};
use tracing::{info, warn};
use types::ai::{
    query::QueryId,
    request::{Role, SignedAiRequest},
    response::SignedAiResponse,
    verification::Verified,
};

pub type NodeResponse = (PeerId, Result<SignedAiResponse, String>);

pub struct Tasks {
    env: Arc<Env>,
    tasks: HashMap<QueryId, Sender<NodeResponse>>,
}

impl Tasks {
    pub fn new(
        query: Queries,
        accounts: Accounts,
        cfg: &AiTasksConfig,
        verifier: Sender<VerificationRequest>,
        etp: ToP2P,
    ) -> Self {
        Self {
            env: Arc::new(Env::new(accounts, query, verifier, cfg.clone(), etp)),
            tasks: HashMap::new(),
        }
    }

    pub async fn new_task(
        &mut self,
        request: Verified<SignedAiRequest>,
        net: &Network,
        tx: oneshot::Sender<Result<QueryId, OrchestratorError>>,
    ) -> Result<(), OrchestratorError> {
        let request = request.into_inner();

        let has_system = request.query.history.iter().any(|h| h.role == Role::System);
        if has_system {
            return Err(OrchestratorError::SystemRoleIsNotAllowed);
        }

        REQUESTS.add(1, &[]);
        PROCESSING.add(1, &[]);

        info!("Handle user request with pubkey: {}", request.query.pubkey);
        let peer_pool = net.connected_peers(self.env.cfg.replication_factor as usize * 4);

        let env = self.env.clone();
        let (task_tx, task_rx) = mpsc::channel(env.cfg.replication_factor as usize);
        let id = env.new_id(&request);
        self.tasks.insert(id, task_tx);

        tokio::task::spawn_blocking(move || match env.new_query(id, request) {
            Ok(query) => {
                let mut task = Task::new(query, peer_pool, env, task_rx);
                tokio::task::spawn(async move {
                    if let Err(err) = task.run(tx).await {
                        ERRORS.add(1, &[]);
                        warn!("Task {:?} failed: {:#?}", task.id(), err);
                    }
                    PROCESSING.add(-1, &[]);
                });
            }
            Err(err) => {
                if tx.send(Err(OrchestratorError::StorageError(err))).is_err() {
                    warn!("Failed to send response to orchestrator");
                }
            }
        });

        Ok(())
    }

    pub async fn on_node_response(
        &mut self,
        id: QueryId,
        sender: PeerId,
        response: Result<SignedAiResponse, String>,
    ) {
        if let Some(task) = self.tasks.get_mut(&id) {
            let result = task.send((sender, response)).await;
            if let Err(err) = result {
                info!("Task {:?} is closed: {:?}", id, err);
            }
        }
    }

    pub fn gc_tasks(&mut self) {
        self.tasks.retain(|_, task| !task.is_closed());
    }
}
