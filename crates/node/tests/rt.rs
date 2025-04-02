use ai::Ai;
use crypto::ed25519::private::PrivateKey;
use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt as _,
};
use multiaddr::Multiaddr;
use node::spawn_node;
use p2p::{
    etp::{FromETP, ToETP},
    key::ToP2P,
};
use std::{sync::Arc, time::Duration};
use types::{
    ai::{query::QueryId, request::SignedAiRequest},
    p2p::{EveMessage, OrchMessage},
};

pub struct AiMock {
    delay: Duration,
}

impl AiMock {
    pub fn new(delay: Duration) -> Self {
        Self { delay }
    }
}

type AiResp = Result<ai::Answer, ai::error::AiError>;

impl Ai for AiMock {
    async fn ask(&self, question: ai::Question) -> AiResp {
        tokio::time::sleep(self.delay).await;

        if question.message == "error" {
            return Err(ai::error::AiError::InternalError);
        }

        Ok(ai::Answer {
            message: format!("ai:{}", question.message),
            tokens: 0,
        })
    }
}

pub async fn start_node() -> Node {
    let ai = Arc::new(AiMock::new(Duration::from_secs(0)));

    let orch = PrivateKey::generate();
    let node_key = PrivateKey::generate();

    let (to_node, out_rx) = futures::channel::mpsc::channel(100);
    let (in_tx, from_node) = futures::channel::mpsc::channel(100);

    let orch_address: Multiaddr = "/ip4/127.0.0.1/udp/0/quic-v1".parse().unwrap();

    spawn_node(
        in_tx,
        out_rx,
        ai,
        orch.public_key(),
        node_key.clone(),
        orch_address.clone(),
    )
    .await
    .unwrap();

    Node {
        to_node,
        from_node,
        orch,
        node_key,
    }
}

pub struct Node {
    pub from_node: Receiver<ToETP<EveMessage>>,
    pub to_node: Sender<FromETP<EveMessage>>,
    pub orch: PrivateKey,
    pub node_key: PrivateKey,
}

impl Node {
    pub async fn send(&mut self, id: QueryId, request: SignedAiRequest) {
        let msg = FromETP::Receive(
            self.orch.public_key().to_p2p().to_peer_id(),
            EveMessage::Orch(OrchMessage::AiRequest { id, request }),
        );
        self.to_node.send(msg).await.unwrap();
    }
}
