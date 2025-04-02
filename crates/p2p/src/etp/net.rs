use super::{
    nodes::{Node, Nodes, State},
    proto::{Caller, ETM},
    requests::{PublishDiagnostic, Requests},
    DeliveryResult, FromETP,
};
use crate::{
    behaviour::EveBehaviour,
    error::EtpError,
    key::{inbox_topic, EvePrivateKey, EvePublicKey, ToP2P as _},
    sys::now_secs,
    Config,
};
use eyre::eyre;
use futures::{
    channel::{mpsc::Sender, oneshot},
    SinkExt,
};
use libp2p::{
    gossipsub::{IdentTopic, Message, TopicHash},
    Multiaddr, PeerId, Swarm,
};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::Duration};
use tracing::{debug, info, warn};

pub struct EtpNet<Msg>
where
    Msg: Serialize + for<'msg> Deserialize<'msg> + Send + Sync + 'static,
{
    peer_id: PeerId,
    pub_key: EvePublicKey,
    from_etp: Sender<FromETP<Msg>>,
    local_address: Vec<Multiaddr>,
    requests: Requests<Msg>,

    ping_interval: Duration,
    ping_timeout: Duration,
}

impl<Msg> EtpNet<Msg>
where
    Msg: Serialize + for<'msg> Deserialize<'msg> + Send + Sync + Debug + 'static,
{
    pub fn new(key: EvePrivateKey, from_etp: Sender<FromETP<Msg>>, cfg: &Config) -> Self {
        let pub_key = key.public_key();
        let peer_id = pub_key.to_p2p().to_peer_id();
        Self {
            from_etp,
            local_address: vec![],
            requests: Requests::new(cfg.request_timeout, inbox_topic(peer_id).hash()),
            pub_key,
            peer_id,
            ping_interval: cfg.ping_interval,
            ping_timeout: cfg.ping_timeout,
        }
    }

    pub fn pub_key(&self) -> &EvePublicKey {
        &self.pub_key
    }

    pub fn inbox(&self) -> IdentTopic {
        inbox_topic(self.pub_key.to_p2p().to_peer_id())
    }

    pub fn new_listen_addr(&mut self, local_address: Multiaddr) {
        self.local_address.push(local_address);
    }

    pub async fn remove_whitelist(
        &mut self,
        key: EvePublicKey,
        swarm: &mut Swarm<EveBehaviour>,
    ) -> Result<(), EtpError> {
        let peer_id = key.to_p2p().to_peer_id();
        if swarm.disconnect_peer_id(peer_id).is_err() {
            warn!("Failed to disconnect peer: {:?}", peer_id);
        }
        Ok(())
    }

    pub async fn send(
        &mut self,
        message: Msg,
        on_received: Option<oneshot::Sender<DeliveryResult>>,
        swarm: &mut Swarm<EveBehaviour>,
        node: &mut Node,
    ) -> Result<(), EtpError> {
        if !node.is_ready() {
            warn!(
                "Failed to send message to non-connected peer: {:?}",
                node.peer()
            );
            if let Some(tx) = on_received {
                let _ = tx.send(DeliveryResult::NotConnected);
            }
            return Ok(());
        }

        if matches!(
            self.requests
                .send_message(message, on_received, swarm, node),
            PublishDiagnostic::CheckConnection
        ) {
            self.try_close_connection(swarm, node).await?;
        }

        Ok(())
    }

    pub fn listeners(&self, sender: oneshot::Sender<Vec<Multiaddr>>) -> Result<(), EtpError> {
        sender
            .send(self.local_address.clone())
            .map_err(|_| EtpError::Common(eyre!("Failed to send listeners to")))
    }

    pub fn dial(
        &mut self,
        address: Multiaddr,
        swarm: &mut Swarm<EveBehaviour>,
        node: &mut Node,
    ) -> Result<(), EtpError> {
        node.set_dial(address.clone());
        if swarm.is_connected(&node.peer()) {
            debug!("Already connected to peer: {:?}", node.peer());
            return Ok(());
        }

        swarm
            .dial(address.clone())
            .map_err(|err| EtpError::DialError(err, address))
    }

    pub(crate) async fn interval(&mut self, swarm: &mut Swarm<EveBehaviour>, nodes: &mut Nodes) {
        self.requests.handle_timeouts().await;

        let now = now_secs();
        for node in nodes.iter_mut() {
            // send ping
            if now - node.last_activity() > self.ping_interval.as_secs()
                && (node.is_ready() || node.is_connected())
            {
                let _ = self.requests.send_ping(swarm, node).await;
            }

            if node.is_ready() {
                continue;
            }

            // check connection
            if node.state() == State::Connected {
                let result = self
                    .requests
                    .send_connected(self.peer_id.into(), swarm, node)
                    .await;
                info!("Checking connection to peer: {:?} {:?}", node, result);
                if result == PublishDiagnostic::CheckConnection && !swarm.is_connected(&node.peer())
                {
                    info!("Disconnecting from peer: {:?}", node.peer());
                    node.disconnect();
                    continue;
                }
            }

            // disconnect peer
            if node.state() != State::Disconnected
                && now - node.last_activity() > self.ping_timeout.as_secs()
            {
                if let Err(err) = self.disconnect(swarm, node).await {
                    warn!("Failed to disconnect peer: {:?}", err);
                }
                continue;
            }

            if let Some(address) = node.dial() {
                if let Err(err) = self.dial(address, swarm, node) {
                    warn!("Failed to dial peer: {:?}", err);
                }
            }
        }
    }

    pub async fn on_message(
        &mut self,
        msg: Message,
        swarm: &mut Swarm<EveBehaviour>,
        node: &mut Node,
    ) -> Result<(), EtpError> {
        let msg = if let Some(msg) = self.requests.on_message(msg).await? {
            msg
        } else {
            return Ok(());
        };

        debug!("Received message from: {:?} {:?}", node, msg);

        if matches!(
            self.requests.send_ack(msg.id, swarm, node).await,
            PublishDiagnostic::CheckConnection
        ) {
            self.try_close_connection(swarm, node).await?;
        }

        match msg.etm {
            ETM::Send(send) => self.handle_etm_send(send, node).await?,
            ETM::Disconnected => self.handle_etm_disconnected(swarm, node).await?,
            ETM::Connected(ready) => self.handle_etm_connected(ready, swarm, node).await?,
            ETM::Ping => node.ping(),
            ETM::Ack(_) => {
                warn!("Unexpected ack message");
            }
            ETM::ReConnect => {
                self.handle_reconnect(swarm, node).await?;
            }
        }
        Ok(())
    }

    pub async fn send_disconnect(
        &mut self,
        swarm: &mut Swarm<EveBehaviour>,
        node: &mut Node,
    ) -> Result<(), EtpError> {
        let _ = self.requests.send_disconnect(swarm, node).await;
        self.disconnect(swarm, node).await
    }

    async fn disconnect(
        &mut self,
        swarm: &mut Swarm<EveBehaviour>,
        node: &mut Node,
    ) -> Result<(), EtpError> {
        info!("Disconnecting from {}", node.peer());
        if swarm.disconnect_peer_id(node.peer()).is_err() {
            debug!("Failed to disconnect peer: {:?}", node.peer());
        }

        let was_ready = node.disconnect();
        if was_ready {
            if let Err(err) = self.from_etp.send(FromETP::Disconnect(node.peer())).await {
                warn!("Failed to send disconnect message: {}", err);
                return Err(EtpError::AppError);
            }
        }
        Ok(())
    }

    pub async fn on_unsubscribed(
        &mut self,
        topic: TopicHash,
        node: &mut Node,
    ) -> Result<(), EtpError> {
        match node.try_unsubscribe(&topic) {
            Ok(_) => {
                if let Err(err) = self.from_etp.send(FromETP::Disconnect(node.peer())).await {
                    warn!("Failed to send disconnect message: {}", err);
                    return Err(EtpError::AppError);
                }
            }
            Err(err) => {
                warn!(
                    "Failed to unsubscribe from topic: {:?} {:?}",
                    node.peer(),
                    err
                );
            }
        }

        Ok(())
    }

    pub async fn on_subscribed(
        &mut self,
        topic: TopicHash,
        swarm: &mut Swarm<EveBehaviour>,
        node: &mut Node,
    ) -> Result<(), EtpError> {
        info!("On subscribed {:?}", node);
        match node.set_ready_on_sub(&topic) {
            Ok(_) => {
                info!("Subscribed to topic: {:?}", node);
                if let Err(err) = self.from_etp.send(FromETP::Connect(node.peer())).await {
                    warn!("Failed to send connect message: {}", err);
                    return Err(EtpError::AppError);
                }
                if matches!(
                    self.requests
                        .send_connected(self.peer_id.into(), swarm, node)
                        .await,
                    PublishDiagnostic::CheckConnection
                ) {
                    {
                        self.try_close_connection(swarm, node).await?;
                    }
                }
            }
            Err(err) => {
                warn!("Failed to subscribe to topic: {:?}", err);
                node.disconnect();
                info!("Disconnecting peer: {:?}", node.peer());
                swarm
                    .disconnect_peer_id(node.peer())
                    .map_err(|_| EtpError::DisconnectError(node.peer()))?;
            }
        }
        Ok(())
    }

    pub async fn try_close_connection(
        &mut self,
        swarm: &mut Swarm<EveBehaviour>,
        node: &mut Node,
    ) -> Result<(), EtpError> {
        info!("Try close connection {:?}", node);
        let state = node.state();
        if state.has_connection() || swarm.is_connected(&node.peer()) {
            self.disconnect(swarm, node).await?;
        }

        Ok(())
    }

    async fn handle_etm_connected(
        &mut self,
        caller: Caller,
        swarm: &mut Swarm<EveBehaviour>,
        node: &mut Node,
    ) -> Result<(), EtpError> {
        if node.is_ready() {
            return Ok(());
        }
        info!("handle_etm_ready peer: {:?}", node);

        let old_state = node.set_ready();

        if old_state != State::Ready {
            self.from_etp
                .send(FromETP::Connect(node.peer()))
                .await
                .map_err(|_| EtpError::AppError)?;
        }

        if caller.as_ref() == &self.peer_id {
            return Ok(());
        }

        let diagnostic = self.requests.send_connected(caller, swarm, node).await;

        if diagnostic == PublishDiagnostic::CheckConnection {
            node.set_state(old_state);
            self.try_close_connection(swarm, node).await?;
            return Ok(());
        }

        Ok(())
    }

    async fn handle_etm_disconnected(
        &mut self,
        swarm: &mut Swarm<EveBehaviour>,
        node: &mut Node,
    ) -> Result<(), EtpError> {
        info!("Disconnected from peer: {:?}", node.peer());
        self.disconnect(swarm, node).await
    }

    async fn handle_etm_send(&mut self, msg: Msg, node: &mut Node) -> Result<(), EtpError> {
        if !node.is_ready() {
            warn!(
                "Received message from non-connected peer: {:?}",
                node.peer()
            );
        }
        node.ping();

        self.from_etp
            .send(FromETP::Receive(node.peer(), msg))
            .await
            .map_err(|_| EtpError::AppError)
    }

    async fn handle_reconnect(
        &mut self,
        swarm: &mut Swarm<EveBehaviour>,
        node: &mut Node,
    ) -> Result<(), EtpError> {
        info!("Reconnecting to peer: {:?}", node.peer());
        self.disconnect(swarm, node).await?;
        if let Some(addr) = node.dial_address() {
            swarm
                .dial(addr.clone())
                .map_err(|err| EtpError::DialError(err, addr))?;
        }
        Ok(())
    }

    pub(crate) async fn on_invalid_state(
        &mut self,
        swarm: &mut Swarm<EveBehaviour>,
        node: &mut Node,
    ) -> Result<(), EtpError> {
        warn!("Invalid state for peer: {:?}", node.peer());
        let _ = self.requests.send_reconnect(swarm, node).await;

        self.handle_reconnect(swarm, node).await
    }
}
