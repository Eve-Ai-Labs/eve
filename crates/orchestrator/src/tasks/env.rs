use crate::{
    store::{accounts, queries},
    verifier::VerificationRequest,
    OrchestratorError, ToP2P,
};
use crypto::ed25519::public::PublicKey;
use futures::{
    channel::oneshot::{self, Receiver},
    SinkExt,
};
use multiaddr::PeerId;
use node_config::tasks::AiTasksConfig;
use p2p::etp::DeliveryResult;
use rand::random;
use tokio::sync::mpsc::Sender;
use types::{
    ai::{
        query::{query_id, Query, QueryId},
        request::SignedAiRequest,
    },
    p2p::OrchMessage,
};

pub struct Env {
    accounts: accounts::Accounts,
    queries: queries::Queries,
    verifier: Sender<VerificationRequest>,
    pub cfg: AiTasksConfig,
    etp: ToP2P,
}

impl Env {
    pub fn new(
        accounts: accounts::Accounts,
        queries: queries::Queries,
        verifier: Sender<VerificationRequest>,
        cfg: AiTasksConfig,
        etp: ToP2P,
    ) -> Self {
        Self {
            accounts,
            queries,
            verifier,
            cfg,
            etp,
        }
    }

    pub fn new_id(&self, req: &SignedAiRequest) -> QueryId {
        query_id(random(), req)
    }

    pub async fn send_to_evaluator(
        &self,
        req: VerificationRequest,
    ) -> Result<(), OrchestratorError> {
        self.verifier
            .send(req)
            .await
            .map_err(|_| OrchestratorError::VerifierError)
    }

    pub fn new_query(
        &self,
        id: QueryId,
        request: SignedAiRequest,
    ) -> Result<Query, storage::StorageError> {
        self.queries.new_query(id, request)
    }

    pub fn update_query(&self, query: &Query) -> Result<(), storage::StorageError> {
        self.queries.update_query(query)
    }

    pub async fn send_request(
        &self,
        peer: PeerId,
        id: QueryId,
        request: SignedAiRequest,
    ) -> Result<Receiver<DeliveryResult>, ()> {
        let (tx, rx) = oneshot::channel();
        let mut etp = self.etp.clone();
        etp.send(p2p::etp::ToETP::Send {
            to: peer,
            message: types::p2p::EveMessage::Orch(OrchMessage::AiRequest { id, request }),
            on_received: Some(tx),
        })
        .await
        .map_err(|_| ())?;

        Ok(rx)
    }

    pub fn transfer(
        &self,
        from: PublicKey,
        to: PublicKey,
        amount: u64,
    ) -> Result<(), OrchestratorError> {
        self.accounts.transfer(from, to, amount)
    }
}
