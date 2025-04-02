use arc_swap::ArcSwap;
use eyre::{eyre, Error};
use orchestrator::ApiSender;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use types::cluster::ClusterInfoWithNodes;

pub struct Cluster {
    sender: ApiSender,
    ttl: Duration,
    info: ArcSwap<(ClusterInfoWithNodes, Instant)>,
    up_lock: Mutex<()>,
}

impl Cluster {
    pub fn new(sender: ApiSender, ttl: Duration) -> Self {
        Self {
            sender,
            ttl,
            info: ArcSwap::new(Arc::new((
                ClusterInfoWithNodes::default(),
                Instant::now() - ttl,
            ))),
            up_lock: Mutex::new(()),
        }
    }

    pub async fn load_info(&self) -> Result<ClusterInfoWithNodes, Error> {
        if let Some(info) = self.load_cached() {
            return Ok(info);
        }

        let _lock = self.up_lock.lock().await;
        if let Some(info) = self.load_cached() {
            return Ok(info);
        }

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(orchestrator::OrchRequest::ClusterInfo { tx })
            .await
            .map_err(|_| eyre!("Failed to send request to orchestrator"))?;

        let info = rx
            .await
            .map_err(|_| eyre!("Failed to receive response from orchestrator"))?;
        self.update_info(info.clone());
        Ok(info)
    }

    fn load_cached(&self) -> Option<ClusterInfoWithNodes> {
        let info_with_inst = self.info.load_full();
        if info_with_inst.1 < Instant::now() {
            return None;
        }
        Some(info_with_inst.0.clone())
    }

    fn update_info(&self, new_info: ClusterInfoWithNodes) {
        self.info
            .store(Arc::new((new_info, Instant::now() + self.ttl)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orchestrator::OrchRequest;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_load_info_cached() {
        let (tx, _rx) = mpsc::channel(1);
        let cluster = Cluster::new(tx, Duration::from_secs(10));

        let test_info = ClusterInfoWithNodes::default();
        cluster.update_info(test_info.clone());

        let result = cluster.load_info().await.unwrap();
        assert_eq!(result, test_info);
    }

    #[tokio::test]
    async fn test_load_info_expired() {
        let (tx, mut rx) = mpsc::channel(1);
        let cluster = Cluster::new(tx, Duration::from_secs(0));

        let test_info = ClusterInfoWithNodes::default();

        // Spawn handler for orchestrator request
        let test_info_inner = test_info.clone();
        tokio::spawn(async move {
            if let Some(OrchRequest::ClusterInfo { tx }) = rx.recv().await {
                tx.send(test_info_inner.clone()).unwrap();
            }
        });

        let result = cluster.load_info().await.unwrap();
        assert_eq!(result, test_info);
    }

    #[tokio::test]
    async fn test_load_info_sender_error() {
        let (tx, _rx) = mpsc::channel(1);
        let cluster = Cluster::new(tx, Duration::from_secs(0));

        // Channel is dropped, should cause error
        drop(_rx);

        let result = cluster.load_info().await;
        assert!(result.is_err());
    }
}
