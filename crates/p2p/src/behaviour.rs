use libp2p::{gossipsub, identity, swarm::NetworkBehaviour};
use std::{
    hash::{DefaultHasher, Hash as _, Hasher as _},
    time::Duration,
};

#[derive(NetworkBehaviour)]
pub struct EveBehaviour {
    pub(super) gossip: gossipsub::Behaviour,
}

impl EveBehaviour {
    pub fn new(key: &identity::Keypair) -> Result<Self, &'static str> {
        let message_id_fn = |message: &gossipsub::Message| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            message.topic.hash(&mut s);
            gossipsub::MessageId::from(s.finish().to_string())
        };

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(5))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .max_transmit_size(usize::MAX)
            .message_id_fn(message_id_fn)
            .build()
            .map_err(|msg| {
                eprintln!("Error creating gossipsub config: {}", msg);
                "Error creating gossipsub config"
            })?;

        let gossip = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(key.clone()),
            gossipsub_config,
        )?;

        Ok(EveBehaviour { gossip })
    }
}
