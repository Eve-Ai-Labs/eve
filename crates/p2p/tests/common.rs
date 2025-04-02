#![allow(dead_code)]
use futures::{channel::mpsc::Sender, SinkExt, StreamExt as _};
use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};
use p2p::{
    etp::{DeliveryResult, FromETP, ToETP},
    key::{EvePrivateKey, EvePublicKey, ToP2P as _},
    spawn, Config,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashSet, VecDeque},
    fmt::Debug,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::sleep;
use tracing::info;

pub struct Node<Msg> {
    key: EvePrivateKey,
    orch: EvePublicKey,
    to_etp: Sender<ToETP<Msg>>,
    addresses: Vec<Multiaddr>,
    inbox: Arc<Mutex<VecDeque<(PeerId, Msg)>>>,
    peers: Arc<Mutex<HashSet<PeerId>>>,
}

impl<Msg> Node<Msg>
where
    Msg: Serialize + for<'msg> Deserialize<'msg> + Send + Sync + Debug + 'static,
{
    pub async fn dial(&self, peer: PeerId, address: Multiaddr) {
        self.to_etp
            .clone()
            .send(ToETP::Dial(peer, address))
            .await
            .unwrap();
    }

    pub fn pub_key(&self) -> EvePublicKey {
        self.key.public_key()
    }

    pub async fn shutdown(self) {
        self.to_etp.clone().send(ToETP::Shutdown).await.unwrap();
    }

    pub fn peer_id(&self) -> PeerId {
        self.key.public_key().to_p2p().to_peer_id()
    }

    pub fn address(&self) -> Vec<Multiaddr> {
        self.addresses.clone()
    }

    pub fn quic_addr(&self) -> Multiaddr {
        self.addresses
            .iter()
            .find(|addr| {
                addr.iter()
                    .any(|p| p == Protocol::QuicV1 || p == Protocol::Quic)
            })
            .unwrap()
            .clone()
    }

    pub fn webrtc_addr(&self) -> Multiaddr {
        self.addresses
            .iter()
            .find(|addr| {
                addr.iter()
                    .any(|p| p == Protocol::WebRTC || p == Protocol::WebRTCDirect)
            })
            .unwrap()
            .clone()
    }

    pub async fn next_msg(&self, block: bool) -> Option<(PeerId, Msg)> {
        if block {
            loop {
                {
                    let mut inbox = self.inbox.lock().unwrap();
                    if !inbox.is_empty() {
                        return inbox.pop_front();
                    }
                }
                sleep(Duration::from_millis(200)).await;
            }
        }
        self.inbox.lock().unwrap().pop_front()
    }

    pub fn orch(&self) -> EvePublicKey {
        self.orch
    }

    pub async fn peers(&self, block: bool) -> HashSet<PeerId> {
        if block {
            loop {
                {
                    let peers = self.peers.lock().unwrap();
                    if !peers.is_empty() {
                        return peers.clone();
                    }
                }
                sleep(Duration::from_millis(200)).await;
            }
        }
        self.peers.lock().unwrap().clone()
    }

    pub async fn whitelist(&self, key: EvePublicKey, addr: Vec<Multiaddr>) {
        self.to_etp
            .clone()
            .send(ToETP::Whitelisted(key, addr))
            .await
            .unwrap();
    }

    pub async fn send(&self, to: PeerId, message: Msg) -> DeliveryResult {
        let (tx, rx) = futures::channel::oneshot::channel();
        self.to_etp
            .clone()
            .send(ToETP::Send {
                to,
                message,
                on_received: Some(tx),
            })
            .await
            .unwrap();
        rx.await.unwrap()
    }

    pub async fn spawn_orch() -> Self {
        Self::spawn(None).await
    }

    pub async fn spawn_node(orch: EvePublicKey) -> Self {
        Self::spawn(Some(orch)).await
    }

    async fn spawn(orch: Option<EvePublicKey>) -> Self {
        let key = EvePrivateKey::generate();
        let address = [
            "/ip4/127.0.0.1/udp/0/quic-v1".parse().unwrap(),
            "/ip4/127.0.0.1/udp/0/webrtc-direct".parse().unwrap(),
        ];
        let orch = orch.unwrap_or_else(|| key.public_key());

        info!("Spawning node with key: {:?}", key.public_key());
        let (_, mut from_etp, mut to_etp) =
            spawn::<Msg>(key.clone(), orch, vec![], &address, Config::default())
                .await
                .unwrap();
        sleep(Duration::from_millis(100)).await;

        let (tx, rx) = futures::channel::oneshot::channel();
        to_etp.send(ToETP::Listeners(tx)).await.unwrap();
        let addresses = rx.await.unwrap();

        let inbox = Arc::new(Mutex::new(VecDeque::new()));
        let peers = Arc::new(Mutex::new(HashSet::new()));

        {
            let inbox = inbox.clone();
            let peers = peers.clone();
            tokio::spawn(async move {
                while let Some(msg) = from_etp.next().await {
                    match msg {
                        FromETP::Receive(peer_id, msg) => {
                            inbox.lock().unwrap().push_back((peer_id, msg));
                        }
                        FromETP::Connect(peer_id) => {
                            peers.lock().unwrap().insert(peer_id);
                        }
                        FromETP::Disconnect(peer_id) => {
                            peers.lock().unwrap().remove(&peer_id);
                        }
                    }
                }
            });
        }

        Self {
            key,
            orch,
            to_etp,
            addresses,
            inbox,
            peers,
        }
    }
}
