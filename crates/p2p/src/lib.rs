use crate::{
    key::{EvePrivateKey, ToP2P},
    task::P2PTask,
};
use etp::{FromETP, ToETP};
use eyre::{Context as _, Error};
use futures::channel::mpsc::{Receiver, Sender};
use key::EvePublicKey;
use libp2p::multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::Duration};
use sys::interval_generator;
use tracing::info;

mod behaviour;
pub mod error;
pub mod etp;
pub mod key;
mod swarm;
pub mod sys;
pub mod task;

pub async fn spawn<Msg>(
    key: EvePrivateKey,
    orch_key: EvePublicKey,
    orch_address: Vec<Multiaddr>,
    address: &[Multiaddr],
    cfg: Config,
) -> Result<(P2PHandler, Receiver<FromETP<Msg>>, Sender<ToETP<Msg>>), Error>
where
    Msg: Serialize + for<'msg> Deserialize<'msg> + Send + Sync + Debug + 'static,
{
    let (p2p_in_tx, p2p_in_rx) = futures::channel::mpsc::channel(100);
    let (p2p_out_tx, p2p_out_rx) = futures::channel::mpsc::channel(100);

    let p2p_key = key.to_p2p();

    let mut swarm = swarm::build_swarm(p2p_key, cfg.connection_timeout)?;

    for addr in address {
        swarm
            .listen_on(addr.clone())
            .context("failed to listen on address")?;
        info!("Start listening on: {}", addr);
    }

    let task = P2PTask::new(p2p_in_tx, orch_key, orch_address, key, &cfg);

    let intervals = interval_generator(cfg.bg_interval);

    #[cfg(not(target_arch = "wasm32"))]
    let handler = tokio::spawn(async move { task.run(p2p_out_rx, intervals, &mut swarm).await });
    #[cfg(target_arch = "wasm32")]
    let handler =
        async_wasm_task::spawn(async move { task.run(p2p_out_rx, intervals, &mut swarm).await });

    Ok((P2PHandler { handler }, p2p_in_rx, p2p_out_tx))
}

pub struct P2PHandler {
    #[cfg(not(target_arch = "wasm32"))]
    pub handler: tokio::task::JoinHandle<()>,
    #[cfg(target_arch = "wasm32")]
    pub handler: async_wasm_task::JoinHandle<()>,
}

pub struct Config {
    pub ping_interval: std::time::Duration,
    pub ping_timeout: std::time::Duration,
    pub request_timeout: std::time::Duration,
    pub connection_timeout: std::time::Duration,
    pub bg_interval: std::time::Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bg_interval: Duration::from_secs(10),
            ping_interval: Duration::from_secs(10),
            ping_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(20),
            request_timeout: Duration::from_secs(10),
        }
    }
}
