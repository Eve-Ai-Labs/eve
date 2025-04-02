use crate::{
    error::EtpError,
    key::{inbox_topic, EvePublicKey, ToP2P as _},
    sys::{is_web, now_secs},
};
use libp2p::{gossipsub::TopicHash, multiaddr::Protocol, Multiaddr, PeerId};
use std::{
    collections::HashMap,
    fmt::{self, Formatter},
};
use tracing::{info, warn};

#[derive(Debug)]
pub(crate) struct Nodes {
    is_orch: bool,
    nodes: HashMap<PeerId, Node>,
}

impl Nodes {
    pub fn new(orch_key: EvePublicKey, orch_address: Vec<Multiaddr>, is_orch: bool) -> Self {
        let mut nodes = HashMap::new();
        let orch_peer_id = orch_key.to_p2p().to_peer_id();
        let mut orch = Node::new(orch_key);
        orch.addresses = orch_address;
        nodes.insert(orch_peer_id, orch);

        Self { is_orch, nodes }
    }

    pub fn get(&mut self, peer_id: PeerId) -> Result<&mut Node, EtpError> {
        self.nodes
            .get_mut(&peer_id)
            .ok_or_else(|| EtpError::NotWhitelisted(peer_id))
    }

    pub fn whitelist(&mut self, key: EvePublicKey, addr: Vec<Multiaddr>) -> Result<(), EtpError> {
        let peer_id = key.to_p2p().to_peer_id();
        if !self.is_orch {
            warn!("Only orchestrator can whitelist peers");
            return Ok(());
        }
        if self.nodes.contains_key(&peer_id) {
            warn!("Peer {} already whitelisted", peer_id);
            return Ok(());
        }

        let mut node = Node::new(key);
        node.addresses = addr;
        self.nodes.insert(peer_id, node);
        info!("Whitelisted peer: {}", peer_id);
        Ok(())
    }

    pub fn remove_whitelist(&mut self, peer_id: PeerId) -> Option<Node> {
        if !self.is_orch {
            warn!("Only orchestrator can remove peers from whitelist");
            return None;
        }

        self.nodes.remove(&peer_id)
    }

    pub fn connected(&mut self) -> impl Iterator<Item = &mut Node> {
        self.nodes
            .iter_mut()
            .filter(|(_, unit)| unit.state.has_connection())
            .map(|(_, node)| node)
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut Node> {
        self.nodes.values_mut()
    }
}

pub struct Node {
    peer_id: PeerId,
    pub_key: EvePublicKey,
    inbox: TopicHash,
    state: State,
    addresses: Vec<Multiaddr>,
    last_activity: u64,
    auto_dial: bool,
}

impl Node {
    pub fn new(key: EvePublicKey) -> Self {
        let peer_id = key.to_p2p().to_peer_id();
        Self {
            peer_id,
            state: State::Disconnected,
            addresses: vec![],
            pub_key: key,
            last_activity: now_secs(),
            inbox: inbox_topic(peer_id).hash(),
            auto_dial: false,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.state == State::Ready
    }

    pub fn is_connected(&self) -> bool {
        self.state == State::Connected
    }

    pub fn touch(&mut self) {
        self.last_activity = now_secs();
    }

    pub fn topic(&self) -> TopicHash {
        self.inbox.clone()
    }

    pub fn peer(&self) -> PeerId {
        self.peer_id
    }

    pub fn key(&self) -> EvePublicKey {
        self.pub_key
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn ping(&mut self) {
        self.touch();
    }

    pub fn last_activity(&self) -> u64 {
        self.last_activity
    }

    pub(crate) fn dial_address(&self) -> Option<Multiaddr> {
        if is_web() {
            self.webrtc_addr()
        } else {
            self.quic_addr()
        }
        .cloned()
    }

    pub(crate) fn try_unsubscribe(&mut self, topic: &TopicHash) -> Result<EvePublicKey, EtpError> {
        if self.state == State::Ready && &self.inbox == topic {
            self.touch();
            self.state = State::Connected;
            Ok(self.pub_key)
        } else {
            Err(EtpError::InvalidState(self.peer_id, self.state))
        }
    }

    pub fn disconnect(&mut self) -> bool {
        let was_ready = self.state == State::Ready;
        self.state = State::Disconnected;
        was_ready
    }

    pub fn quic_addr(&self) -> Option<&Multiaddr> {
        self.addresses.iter().find(|addr| {
            addr.iter()
                .any(|p| p == Protocol::QuicV1 || p == Protocol::Quic)
        })
    }

    pub fn webrtc_addr(&self) -> Option<&Multiaddr> {
        self.addresses.iter().find(|addr| {
            addr.iter()
                .any(|p| p == Protocol::WebRTC || p == Protocol::WebRTCDirect)
        })
    }

    pub(crate) fn set_dial(&mut self, address: Multiaddr) {
        self.auto_dial = true;
        if !self.addresses.contains(&address) {
            self.addresses.push(address);
        }
    }

    pub(crate) fn try_connect(&mut self) -> bool {
        let was_connected = self.state.has_connection();
        self.state = State::Connected;
        !was_connected
    }

    pub fn set_ready_on_sub(&mut self, topic: &TopicHash) -> Result<(), EtpError> {
        if self.state == State::Connected && &self.inbox == topic {
            self.touch();
            self.state = State::Ready;
            Ok(())
        } else {
            Err(EtpError::InvalidState(self.peer_id, self.state))
        }
    }

    pub(crate) fn set_state(&mut self, state: State) {
        self.state = state;
    }

    pub(crate) fn set_ready(&mut self) -> State {
        let old_state = self.state;
        self.state = State::Ready;
        self.touch();
        old_state
    }

    pub(crate) fn dial(&self) -> Option<Multiaddr> {
        if self.state == State::Disconnected && self.auto_dial {
            if is_web() {
                self.webrtc_addr().cloned()
            } else {
                self.quic_addr().cloned()
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Disconnected,
    Connected,
    Ready,
}

impl State {
    pub fn has_connection(&self) -> bool {
        matches!(self, State::Connected | State::Ready)
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.peer_id, self.state)
    }
}
