use crate::{error::NodeError, ToP2P};
use crypto::ed25519::public::PublicKey;
use events::node::{send_node_status_event, NodeStatus};
use futures::SinkExt as _;
use multiaddr::Multiaddr;
use p2p::{key::ToP2P as _, task::PeerId};
use tracing::{info, warn};

pub struct Network {
    orch: Node,
    p2p_sender: ToP2P,
}

impl Network {
    pub fn new(orch_address: Multiaddr, orch_public_key: PublicKey, p2p_sender: ToP2P) -> Self {
        Self {
            orch: Node::new(orch_public_key.to_p2p().to_peer_id(), Some(orch_address)),
            p2p_sender,
        }
    }

    pub fn is_orch(&self, peer_id: PeerId) -> bool {
        self.orch.peer_id == peer_id
    }

    pub async fn disconnect_peer(&mut self, peer_id: PeerId) -> Result<(), NodeError> {
        if self.orch.peer_id == peer_id {
            if !self.orch.is_connected() {
                info!("Orchestrator is already disconnected");
            }
            send_node_status_event(NodeStatus::Offline)?;
            self.orch.set_connected(false);
            info!("Disconnecting from orchestrator");
        } else {
            warn!("Disconnecting from node {peer_id}");
        }
        Ok(())
    }

    pub async fn connect_peer(&mut self, peer_id: PeerId) -> Result<(), NodeError> {
        if self.orch.peer_id == peer_id {
            if !self.orch.is_connected() {
                send_node_status_event(NodeStatus::Online)?;
                self.orch.set_connected(true);
                info!("Connected to orchestrator");
            } else {
                warn!("Orchestrator is already connected");
            }
        } else {
            warn!("Connecting to node {peer_id}");
        }
        Ok(())
    }

    pub async fn reconnect_nodes(&mut self) -> Result<(), NodeError> {
        self.orch.dial(&mut self.p2p_sender).await?;
        Ok(())
    }
}

#[derive(Debug)]
struct Node {
    connected: bool,
    peer_id: PeerId,
    address: Option<Multiaddr>,
}

impl Node {
    fn new(peer_id: PeerId, address: Option<Multiaddr>) -> Self {
        Self {
            connected: false,
            peer_id,
            address,
        }
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }

    async fn dial(&self, p2p_sender: &mut ToP2P) -> Result<(), NodeError> {
        if self.is_connected() {
            return Ok(());
        }
        if let Some(address) = self.address.clone() {
            info!("Dialing: {:?}", address);
            p2p_sender
                .send(p2p::etp::ToETP::Dial(self.peer_id, address))
                .await
                .map_err(|_| NodeError::P2PError)
        } else {
            Ok(())
        }
    }
}
