use super::{
    proto::{Caller, EtmType, MessageId, ProtocolMessage, ETM},
    DeliveryResult,
};
use crate::{behaviour::EveBehaviour, error::EtpError, sys::now_secs};
use futures::channel::oneshot;
use libp2p::gossipsub::{Message, TopicHash};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};
use tracing::{debug, warn};

pub struct Requests<Msg> {
    _msg: std::marker::PhantomData<Msg>,
    requests: HashMap<MessageId, Request>,
    timeout: std::time::Duration,
    inbox: TopicHash,
}

impl<Msg> Requests<Msg>
where
    Msg: Serialize + for<'msg> Deserialize<'msg> + Debug + Send + Sync + 'static,
{
    pub fn new(timeout: std::time::Duration, inbox: TopicHash) -> Self {
        Self {
            _msg: std::marker::PhantomData,
            requests: HashMap::new(),
            timeout,
            inbox,
        }
    }

    #[must_use]
    fn send_etm(
        &mut self,
        etm: ETM<Msg>,
        on_received: Option<oneshot::Sender<DeliveryResult>>,
        swarm: &mut libp2p::Swarm<EveBehaviour>,
        node: &super::nodes::Node,
    ) -> PublishDiagnostic {
        let tp = etm.tp();
        let message = ProtocolMessage::new(node.key(), etm);
        let id = message.id.clone();
        let msg = match bincode::serialize(&message) {
            Ok(msg) => msg,
            Err(err) => {
                warn!("Failed to serialize message: {}", err);
                if let Some(on_received) = on_received {
                    let _ = on_received.send(DeliveryResult::Bincode(err));
                }
                return PublishDiagnostic::None;
            }
        };

        let gossip = &mut swarm.behaviour_mut().gossip;
        if let Err(err) = gossip.publish(node.topic(), msg) {
            warn!("Failed to publish message: {}", err);
            if !swarm.is_connected(&node.peer()) {
                return PublishDiagnostic::NotConnected;
            }

            let diagnostic = match &err {
                libp2p::gossipsub::PublishError::TransformFailed(_)
                | libp2p::gossipsub::PublishError::MessageTooLarge
                | libp2p::gossipsub::PublishError::Duplicate
                | libp2p::gossipsub::PublishError::SigningError(_) => {
                    PublishDiagnostic::InvalidMessage
                }
                libp2p::gossipsub::PublishError::NoPeersSubscribedToTopic => {
                    PublishDiagnostic::NoPeersSubscribedToTopic
                }
                libp2p::gossipsub::PublishError::AllQueuesFull(_) => {
                    PublishDiagnostic::CheckConnection
                }
            };

            if let Some(on_received) = on_received {
                let _ = on_received.send(DeliveryResult::PublishError(err));
            }
            return diagnostic;
        };

        if let Some(on_received) = on_received {
            self.requests.insert(
                id,
                Request {
                    on_received,
                    tp,
                    sent: now_secs(),
                },
            );
        }
        PublishDiagnostic::None
    }

    pub(crate) async fn handle_timeouts(&mut self) {
        let now = now_secs();
        let timeout = self.timeout.as_secs();

        let mut to_remove = Vec::new();
        for (id, request) in &self.requests {
            if now - request.sent > timeout {
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            if let Some(request) = self.requests.remove(&id) {
                let _ = request.on_received.send(DeliveryResult::Timeout);
            }
        }
    }

    async fn acr(&mut self, id: &MessageId) -> Result<(), EtpError> {
        if let Some(request) = self.requests.remove(id) {
            debug!("Received ack for message: {:?} {:?}", id, request.tp);
            let _ = request.on_received.send(DeliveryResult::Success);
        }
        Ok(())
    }

    pub(crate) async fn on_message(
        &mut self,
        msg: Message,
    ) -> Result<Option<ProtocolMessage<Msg>>, EtpError> {
        if msg.topic != self.inbox {
            warn!("Received message for unknown topic: {}", msg.topic);
            return Err(EtpError::UnknownTopic);
        }

        let message: ProtocolMessage<Msg> = match bincode::deserialize(&msg.data) {
            Ok(message) => message,
            Err(err) => {
                warn!("Failed to deserialize message: {}", err);
                return Err(EtpError::Common(err.into()));
            }
        };

        if let Some(ack) = message.etm.as_ack() {
            self.acr(ack).await?;
            return Ok(None);
        }
        Ok(Some(message))
    }

    #[must_use]
    pub(crate) fn send_message(
        &mut self,
        message: Msg,
        on_received: Option<oneshot::Sender<DeliveryResult>>,
        swarm: &mut libp2p::Swarm<EveBehaviour>,
        node: &super::nodes::Node,
    ) -> PublishDiagnostic {
        self.send_etm(ETM::Send(message), on_received, swarm, node)
    }

    #[must_use]
    pub(crate) async fn send_ack(
        &mut self,
        id: MessageId,
        swarm: &mut libp2p::Swarm<EveBehaviour>,
        node: &super::nodes::Node,
    ) -> PublishDiagnostic {
        self.send_etm(ETM::Ack(id), None, swarm, node)
    }

    #[must_use]
    pub(crate) async fn send_connected(
        &mut self,
        ready: Caller,
        swarm: &mut libp2p::Swarm<EveBehaviour>,
        node: &super::nodes::Node,
    ) -> PublishDiagnostic {
        self.send_etm(ETM::Connected(ready), None, swarm, node)
    }

    #[must_use]
    pub(crate) async fn send_reconnect(
        &mut self,
        swarm: &mut libp2p::Swarm<EveBehaviour>,
        node: &super::nodes::Node,
    ) -> PublishDiagnostic {
        self.send_etm(ETM::ReConnect, None, swarm, node)
    }

    #[must_use]
    pub(crate) async fn send_ping(
        &mut self,
        swarm: &mut libp2p::Swarm<EveBehaviour>,
        node: &super::nodes::Node,
    ) -> PublishDiagnostic {
        self.send_etm(ETM::Ping, None, swarm, node)
    }

    pub(crate) async fn send_disconnect(
        &mut self,
        swarm: &mut libp2p::Swarm<EveBehaviour>,
        node: &super::nodes::Node,
    ) -> PublishDiagnostic {
        self.send_etm(ETM::Disconnected, None, swarm, node)
    }
}

#[derive(Debug)]
pub struct Request {
    on_received: oneshot::Sender<DeliveryResult>,
    tp: EtmType,
    sent: u64,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum PublishDiagnostic {
    None,
    CheckConnection,
    InvalidMessage,
    NoPeersSubscribedToTopic,
    NotConnected,
}
