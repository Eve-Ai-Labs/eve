use crate::{
    behaviour::{EveBehaviour, EveBehaviourEvent},
    error::EtpError,
    etp::{net::EtpNet, nodes::Nodes, FromETP, ToETP},
    key::{EvePrivateKey, EvePublicKey, ToP2P as _},
    Config,
};
use futures::{
    channel::mpsc::{Receiver, Sender},
    StreamExt as _,
};
pub use libp2p::PeerId;
use libp2p::{swarm::SwarmEvent, Multiaddr, Swarm};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

pub(crate) struct P2PTask<Msg>
where
    Msg: Serialize + for<'msg> Deserialize<'msg> + Send + Sync + 'static,
{
    etp: EtpNet<Msg>,
    nodes: Nodes,
}

impl<Msg> P2PTask<Msg>
where
    Msg: Serialize + for<'msg> Deserialize<'msg> + Send + Sync + 'static + std::fmt::Debug,
{
    pub(crate) fn new(
        from_etp: Sender<FromETP<Msg>>,
        orch_key: EvePublicKey,
        orch_address: Vec<Multiaddr>,
        key: EvePrivateKey,
        cfg: &Config,
    ) -> Self {
        let is_orch = key.public_key() == orch_key;
        Self {
            etp: EtpNet::new(key, from_etp, cfg),
            nodes: Nodes::new(orch_key, orch_address, is_orch),
        }
    }

    async fn on_swarm_event(
        &mut self,
        event: SwarmEvent<EveBehaviourEvent>,
        swarm: &mut Swarm<EveBehaviour>,
    ) -> Result<(), EtpError> {
        match event {
            SwarmEvent::Behaviour(EveBehaviourEvent::Gossip(
                libp2p::gossipsub::Event::Message {
                    message_id: _,
                    message,
                    ..
                },
            )) => {
                debug!("Received gossipsub message:{:?}", message.source);
                let node = self
                    .nodes
                    .get(message.source.ok_or_else(|| EtpError::UnknownSender)?)?;
                self.etp.on_message(message, swarm, node).await
            }
            SwarmEvent::Behaviour(EveBehaviourEvent::Gossip(
                libp2p::gossipsub::Event::Subscribed { peer_id, topic },
            )) => {
                info!("Subscribed to topic: {:?}{:?}", peer_id, topic);
                self.etp
                    .on_subscribed(topic, swarm, self.nodes.get(peer_id)?)
                    .await
            }
            SwarmEvent::Behaviour(EveBehaviourEvent::Gossip(
                libp2p::gossipsub::Event::Unsubscribed { peer_id, topic },
            )) => {
                info!("Unsubscribed from topic: {:?}{:?}", peer_id, topic);
                self.etp
                    .on_unsubscribed(topic, self.nodes.get(peer_id)?)
                    .await
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connection established with: {:?}", peer_id);
                if let Ok(node) = self.nodes.get(peer_id) {
                    if node.try_connect() {
                        info!("Connected to peer: {:?}", node.peer());
                    } else {
                        debug!("Peer already connected: {:?}", node.peer());
                    }
                } else {
                    let _ = swarm.disconnect_peer_id(peer_id);
                }
                Ok(())
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                info!("Connection closed with: {:?}[{:?}]", peer_id, cause);
                let node = self.nodes.get(peer_id)?;
                self.etp.try_close_connection(swarm, node).await
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on: {}", address);
                self.etp.new_listen_addr(address);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn on_etp_event(
        &mut self,
        event: ToETP<Msg>,
        swarm: &mut Swarm<EveBehaviour>,
    ) -> Result<(), EtpError> {
        match event {
            ToETP::Whitelisted(key, multiaddr) => {
                info!("Whitelisted peer: {:?}", key);
                self.nodes.whitelist(key, multiaddr)
            }
            ToETP::RemoveFromWhitelist(key) => {
                info!("Removing peer from whitelist: {:?}", key);
                let peer_id = key.to_p2p().to_peer_id();
                if let Some(mut node) = self.nodes.remove_whitelist(peer_id) {
                    self.etp.send_disconnect(swarm, &mut node).await?;
                }
                Ok(())
            }
            ToETP::Send {
                to,
                message,
                on_received,
            } => {
                debug!("Sending message to: {:?}", to);
                self.etp
                    .send(message, on_received, swarm, self.nodes.get(to)?)
                    .await
            }
            ToETP::Listeners(sender) => self.etp.listeners(sender),
            ToETP::Dial(peer, address) => {
                info!("Dialing address: {:?} {:?}", peer, address);
                self.etp.dial(address, swarm, self.nodes.get(peer)?)
            }
            ToETP::Shutdown => {
                info!("Shutting down p2p task");

                for node in self.nodes.connected() {
                    self.etp.send_disconnect(swarm, node).await?;
                }

                Err(EtpError::AppError)
            }
        }
    }

    pub async fn handle_error(&mut self, err: EtpError, swarm: &mut Swarm<EveBehaviour>) -> bool {
        warn!("Error handling event: {}", err);
        if err.is_app_error() {
            error!("Shutting down p2p task.");
            return true;
        }
        if let Some(peer) = err.is_invalid_state() {
            warn!("Invalid state.");
            if let Ok(node) = self.nodes.get(peer) {
                if let Err(err) = self.etp.on_invalid_state(swarm, node).await {
                    warn!("Error handling invalid state: {}", err);
                }
            }
        }
        false
    }

    pub(crate) async fn run(
        mut self,
        mut to_etp: Receiver<ToETP<Msg>>,
        mut intervals: Receiver<()>,
        swarm: &mut Swarm<EveBehaviour>,
    ) {
        if let Err(err) = swarm.behaviour_mut().gossip.subscribe(&self.etp.inbox()) {
            error!("Failed to subscribe to gossip: {}", err);
            return;
        }

        info!("Starting P2P task");
        loop {
            futures::select! {
                complete => break,
                _ = intervals.select_next_some() => {
                     self.etp.interval(swarm, &mut self.nodes).await;
                },
                event = to_etp.select_next_some() =>{
                    if let Err(err) = self.on_etp_event(event, swarm).await {
                        if self.handle_error(err, swarm).await {
                            break;
                        }
                    }
                },
                event = swarm.select_next_some() => {
                    if let Err(err) = self.on_swarm_event(event, swarm).await {
                         if self.handle_error(err, swarm).await {
                            break;
                         }
                    }
            }
            }
        }
    }
}
