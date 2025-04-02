use crate::{error::OrchestratorError, ToP2P};
use crypto::ed25519::public::PublicKey;
use futures::{channel::oneshot, SinkExt as _};
use multiaddr::{Multiaddr, Protocol};
use p2p::{etp::ToETP, key::ToP2P as _, task::PeerId};
use rand::seq::IteratorRandom;
use std::{collections::HashMap, sync::Arc};
use storage::{EveStorage, WriteSet};
use tracing::{info, warn};
use types::{
    cluster::{ClusterInfo, ClusterInfoWithNodes, Node},
    p2p::Peer,
};

pub struct Network {
    peers: HashMap<PeerId, Node>,
    connected: Vec<ConnectedNode>,
    p2p: ToP2P,
    storage: Arc<EveStorage>,
    info: Info,
}

impl Network {
    pub fn new(
        self_key: PublicKey,
        p2p: ToP2P,
        storage: Arc<EveStorage>,
    ) -> Result<Self, OrchestratorError> {
        let nodes = storage.cluster_table.nodes()?;

        let peers = nodes
            .iter()
            .filter(|node| node.public_key != self_key)
            .map(|node| {
                let id = node.public_key.to_p2p().to_peer_id();
                (
                    id,
                    Node::new(
                        node.public_key,
                        node.public_key.to_p2p().to_peer_id(),
                        node.address.clone(),
                    ),
                )
            })
            .collect();

        Ok(Self {
            peers,
            p2p,
            storage,
            info: Info::new(self_key),
            connected: vec![],
        })
    }

    pub async fn add_to_cluster(
        &mut self,
        public_key: PublicKey,
        address: Option<Multiaddr>,
    ) -> Result<(), OrchestratorError> {
        let id = public_key.to_p2p().to_peer_id();
        let has_peer = self.peers.iter().any(|(_, node)| {
            node.key == public_key || (node.address.is_some() && node.address == address)
        });
        if has_peer {
            return Err(OrchestratorError::NodeIsAlreadyInWhitelist(
                public_key.to_p2p().to_peer_id(),
            ));
        }

        let storage = self.storage.clone();
        let address_clone = address.clone();
        let persist_node = tokio::task::spawn_blocking(move || {
            let mut ws = WriteSet::default();

            let persist_node = Peer {
                address: address_clone,
                public_key,
            };
            storage.cluster_table.add_node(&persist_node, &mut ws)?;
            storage.commit(ws)?;

            Ok::<_, OrchestratorError>(persist_node)
        })
        .await
        .map_err(|err| OrchestratorError::EyreError(err.into()))??;

        let node = Node::new(
            public_key,
            public_key.to_p2p().to_peer_id(),
            address.clone(),
        );

        self.peers.insert(id, node);
        info!("Node {} added to cluster", public_key);

        if let Some(address) = persist_node.address.clone() {
            self.p2p
                .send(ToETP::Dial(
                    persist_node.public_key.to_p2p().to_peer_id(),
                    address,
                ))
                .await
                .map_err(|_| OrchestratorError::P2PError)?;
        }

        self.p2p
            .send(ToETP::Whitelisted(
                public_key,
                persist_node.address.into_iter().collect(),
            ))
            .await
            .map_err(|_| OrchestratorError::P2PError)?;

        Ok(())
    }

    pub async fn remove_from_cluster(
        &mut self,
        public_key: PublicKey,
    ) -> Result<(), OrchestratorError> {
        let peer = public_key.to_p2p().to_peer_id();

        self.peers
            .remove(&peer)
            .ok_or(OrchestratorError::NodeIsNotInWhitelist(peer))?;

        let storage = self.storage.clone();
        tokio::task::spawn_blocking(move || {
            let mut ws = WriteSet::default();
            storage.cluster_table.remove_node(&public_key, &mut ws)?;
            storage.commit(ws)?;
            Ok::<_, OrchestratorError>(())
        })
        .await
        .map_err(|err| OrchestratorError::EyreError(err.into()))??;

        self.p2p
            .send(ToETP::RemoveFromWhitelist(public_key))
            .await
            .map_err(|_| OrchestratorError::P2PError)?;

        info!("Node {} removed from cluster", public_key);
        Ok(())
    }

    pub fn connected_peers(&self, amount: usize) -> Vec<ConnectedNode> {
        if self.connected.len() < amount {
            return self.connected.clone();
        }

        self.connected
            .iter()
            .choose_multiple(&mut rand::thread_rng(), amount)
            .into_iter()
            .cloned()
            .collect()
    }

    pub fn connect_peer(&mut self, peer: PeerId) -> Result<(), OrchestratorError> {
        let node = self.peers.get_mut(&peer);
        if let Some(node) = node {
            if !node.is_connected() {
                info!("Connected to node {}", peer);
            } else {
                info!("Node {} is already connected", peer);
                return Ok(());
            }
            self.connected
                .push(ConnectedNode::new(node.peer_id, node.key));
            node.set_connected(true);
        } else {
            warn!("Node {} is not in whitelist", peer);
        }
        Ok(())
    }

    pub fn disconnect_peer(&mut self, peer: PeerId) -> Result<(), OrchestratorError> {
        let node = self
            .peers
            .get_mut(&peer)
            .ok_or(OrchestratorError::NodeIsNotInWhitelist(peer))?;

        if !node.is_connected() {
            warn!("Node {} is already disconnected", peer);
            return Ok(());
        }

        self.connected.retain(|node| node.peer_id != peer);
        node.set_connected(false);
        info!("Disconnected from node {}", peer);
        Ok(())
    }

    pub async fn cluster_info(&mut self) -> Result<ClusterInfoWithNodes, OrchestratorError> {
        if !self.has_listen_addresses() {
            let (tx, rx) = oneshot::channel();
            self.p2p
                .send(ToETP::Listeners(tx))
                .await
                .map_err(|_| OrchestratorError::P2PError)?;
            let addresses = rx.await.map_err(|_| OrchestratorError::P2PError)?;
            for addr in addresses {
                self.info.add_address(addr);
            }
        }

        Ok(ClusterInfoWithNodes {
            cluster_info: ClusterInfo {
                orch_address: self.info.addresses.clone(),
                webrtc_certhash: self.info.webrtc_cert_hash.clone(),
                orch_pubkey: self.info.self_key,
                nodes_count: self.peers.len(),
            },
            nodes: self.peers.clone(),
        })
    }

    pub fn has_listen_addresses(&self) -> bool {
        !self.info.addresses.is_empty()
    }

    pub(crate) async fn init_whitelist(&mut self) -> Result<(), OrchestratorError> {
        info!("Initializing whitelist");

        let nodes = self.storage.cluster_table.nodes()?;
        for node in nodes {
            self.p2p
                .send(ToETP::Whitelisted(
                    node.public_key,
                    node.address.into_iter().collect(),
                ))
                .await
                .map_err(|_| OrchestratorError::P2PError)?;
        }

        Ok(())
    }
}

pub struct Info {
    pub addresses: Vec<Multiaddr>,
    pub webrtc_cert_hash: Option<String>,
    pub self_key: PublicKey,
}

impl Info {
    fn new(self_key: PublicKey) -> Self {
        Self {
            addresses: vec![],
            webrtc_cert_hash: None,
            self_key,
        }
    }

    pub fn add_address(&mut self, address: Multiaddr) {
        if self.addresses.contains(&address) {
            return;
        }

        if let Some(certhash) = parse_webrtc_certhash(&address) {
            self.webrtc_cert_hash = Some(certhash);
        }
        self.addresses.push(address);
    }
}

pub fn parse_webrtc_certhash(addr: &Multiaddr) -> Option<String> {
    let mut is_web_rtc = false;
    for part in addr.iter() {
        if matches!(&part, Protocol::WebRTCDirect) {
            is_web_rtc = true;
        }
        if let Protocol::Certhash(hash) = &part {
            if is_web_rtc {
                return Some(hex::encode(hash.to_bytes()));
            }
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct ConnectedNode {
    pub peer_id: PeerId,
    pub key: PublicKey,
}

impl ConnectedNode {
    pub fn new(peer_id: PeerId, key: PublicKey) -> Self {
        Self { peer_id, key }
    }
}
