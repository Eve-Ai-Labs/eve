use common::Node;
use p2p::key::EvePrivateKey;
use tokio::time::sleep;
use tracing_test::traced_test;
mod common;

#[traced_test]
#[tokio::test]
async fn test_connection() {
    let orch = Node::<String>::spawn_orch().await;
    let node = Node::<String>::spawn_node(orch.orch()).await;
    orch.whitelist(node.pub_key(), node.address()).await;

    node.dial(orch.peer_id(), orch.quic_addr()).await;

    assert!(node.peers(true).await.contains(&orch.peer_id()));
    assert!(orch.peers(true).await.contains(&node.peer_id()));

    let node2 = Node::<String>::spawn_node(orch.orch()).await;
    orch.whitelist(node2.pub_key(), node2.address()).await;
    node2.dial(orch.peer_id(), orch.quic_addr()).await;

    assert!(node2.peers(true).await.contains(&orch.peer_id()));
}

#[traced_test]
#[tokio::test]
async fn test_unwhitelisted_connection_fails() {
    let orch = Node::<String>::spawn_orch().await;
    let node = Node::<String>::spawn_node(orch.orch()).await;

    node.dial(orch.peer_id(), orch.quic_addr()).await;
    sleep(std::time::Duration::from_millis(100)).await;
    assert!(!orch.peers(false).await.contains(&node.peer_id()));
    assert!(!node.peers(false).await.contains(&orch.peer_id()));
}

#[traced_test]
#[tokio::test]
async fn test_with_wrong_orch() {
    let orch = Node::<String>::spawn_orch().await;
    let node = Node::<String>::spawn_node(EvePrivateKey::generate().public_key()).await;

    node.dial(orch.peer_id(), orch.quic_addr()).await;

    sleep(std::time::Duration::from_millis(100)).await;
    assert!(!orch.peers(false).await.contains(&node.peer_id()));
    assert!(!node.peers(false).await.contains(&orch.peer_id()));
}

#[traced_test]
#[tokio::test]
async fn test_disconnect() {
    let orch = Node::<String>::spawn_orch().await;
    let node = Node::<String>::spawn_node(orch.orch()).await;
    orch.whitelist(node.pub_key(), node.address()).await;

    node.dial(orch.peer_id(), orch.quic_addr()).await;
    assert!(orch.peers(true).await.contains(&node.peer_id()));
    assert!(node.peers(true).await.contains(&orch.peer_id()));

    node.shutdown().await;

    loop {
        sleep(std::time::Duration::from_millis(400)).await;
        if orch.peers(false).await.is_empty() {
            break;
        }
    }
}

#[traced_test]
#[tokio::test]
async fn test_disconnect_webrtc() {
    let orch = Node::<String>::spawn_orch().await;
    let node = Node::<String>::spawn_node(orch.orch()).await;
    orch.whitelist(node.pub_key(), node.address()).await;

    node.dial(orch.peer_id(), orch.quic_addr()).await;

    assert!(orch.peers(true).await.contains(&node.peer_id()));
    assert!(node.peers(true).await.contains(&orch.peer_id()));

    node.shutdown().await;
    loop {
        sleep(std::time::Duration::from_millis(400)).await;
        if orch.peers(false).await.is_empty() {
            break;
        }
    }
}
