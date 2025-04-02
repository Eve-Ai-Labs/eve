use crate::behaviour::EveBehaviour;
use eyre::Error;
use libp2p::identity::Keypair;
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
pub fn build_swarm(
    key: Keypair,
    connection_timeout: Duration,
) -> Result<libp2p::Swarm<EveBehaviour>, Error> {
    use libp2p::{
        core::{muxing::StreamMuxerBox, Transport},
        futures::future::Either,
    };
    use rand::thread_rng;

    let swarm = libp2p::SwarmBuilder::with_existing_identity(key)
        .with_tokio()
        .with_other_transport(|id_keys| {
            let quic_transport =
                libp2p::quic::tokio::Transport::new(libp2p::quic::Config::new(id_keys));
            let rtc = libp2p_webrtc::tokio::Transport::new(
                id_keys.clone(),
                libp2p_webrtc::tokio::Certificate::generate(&mut thread_rng())?,
            );
            Ok(quic_transport
                .or_transport(rtc)
                .map(|either, _| match either {
                    Either::Left((peer_id, conn)) => (peer_id, StreamMuxerBox::new(conn)),
                    Either::Right((peer_id, conn)) => (peer_id, StreamMuxerBox::new(conn)),
                }))
        })?
        .with_behaviour(|key| Ok(EveBehaviour::new(key)?))?
        .with_swarm_config(|c| c.with_idle_connection_timeout(connection_timeout))
        .build();
    Ok(swarm)
}

#[cfg(target_arch = "wasm32")]
pub fn build_swarm(
    key: Keypair,
    connection_timeout: Duration,
) -> Result<libp2p::Swarm<EveBehaviour>, Error> {
    Ok(libp2p::SwarmBuilder::with_existing_identity(key)
        .with_wasm_bindgen()
        .with_other_transport(|key| {
            libp2p_webrtc_websys::Transport::new(libp2p_webrtc_websys::Config::new(&key))
        })?
        .with_behaviour(|key| Ok(EveBehaviour::new(key)?))?
        .with_swarm_config(|c| c.with_idle_connection_timeout(connection_timeout))
        .build())
}
