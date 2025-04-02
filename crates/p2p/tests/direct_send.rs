mod common;

use common::Node;
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_direct_send() {
    let orch = Node::<String>::spawn_orch().await;
    let node = Node::<String>::spawn_node(orch.orch()).await;
    orch.whitelist(node.pub_key(), node.address()).await;

    node.dial(orch.peer_id(), orch.quic_addr()).await;

    assert!(node.peers(true).await.contains(&orch.peer_id()));
    assert!(orch.peers(true).await.contains(&node.peer_id()));

    node.send(orch.peer_id(), "hello".to_string()).await;
    orch.send(node.peer_id(), "world".to_string()).await;

    assert_eq!(
        orch.next_msg(true).await.unwrap(),
        (node.peer_id(), "hello".to_string())
    );
    assert_eq!(
        node.next_msg(true).await.unwrap(),
        (orch.peer_id(), "world".to_string())
    );

    for i in 0..50 {
        node.send(orch.peer_id(), format!("o-{}", i)).await;
        orch.send(node.peer_id(), format!("n-{}", i)).await;
    }

    for i in 0..50 {
        assert_eq!(
            orch.next_msg(true).await.unwrap(),
            (node.peer_id(), format!("o-{}", i))
        );
        assert_eq!(
            node.next_msg(true).await.unwrap(),
            (orch.peer_id(), format!("n-{}", i))
        );
    }
}

#[traced_test]
#[tokio::test]
async fn test_send_to_disconnected_peer() {
    let orch = Node::<String>::spawn_orch().await;
    let node = Node::<String>::spawn_node(orch.orch()).await;
    orch.whitelist(node.pub_key(), node.address()).await;
    let result = orch.send(node.peer_id(), "hello".to_string()).await;
    assert!(result.is_not_connected());

    node.dial(orch.peer_id(), orch.quic_addr()).await;
    assert!(node.peers(true).await.contains(&orch.peer_id()));
    assert!(orch.peers(true).await.contains(&node.peer_id()));

    let result = orch.send(node.peer_id(), "hello".to_string()).await;
    assert!(result.is_success());

    let node_peer_id = node.peer_id();
    node.shutdown().await;
    let result = orch.send(node_peer_id, "hello".to_string()).await;
    assert!(result.is_error());
}
