mod common;

use crypto::ed25519::private::PrivateKey;
use types::p2p::Peer;

#[test]
pub fn test_cluster_is_empty() {
    let (_, store) = common::test_storage();
    assert!(store.cluster_table.is_empty().unwrap());
    assert!(store.cluster_table.nodes().unwrap().is_empty());
    let mut ws = storage::WriteSet::default();
    store
        .cluster_table
        .add_node(
            &Peer {
                address: Some("/ip4/192.168.0.1".parse().unwrap()),
                public_key: PrivateKey::generate().public_key(),
            },
            &mut ws,
        )
        .unwrap();
    store.commit(ws).unwrap();
    assert!(!store.cluster_table.is_empty().unwrap());
}

#[test]
pub fn test_add_nodes() {
    let (_, store) = common::test_storage();

    let mut ws = storage::WriteSet::default();
    let node_1 = Peer {
        address: Some("/ip4/192.168.0.1".parse().unwrap()),
        public_key: PrivateKey::generate().public_key(),
    };
    store.cluster_table.add_node(&node_1, &mut ws).unwrap();

    let node_2 = Peer {
        address: Some("/ip4/192.168.0.2".parse().unwrap()),
        public_key: PrivateKey::generate().public_key(),
    };
    store.cluster_table.add_node(&node_2, &mut ws).unwrap();
    store.commit(ws).unwrap();

    let nodes = store.cluster_table.nodes().unwrap();

    assert_eq!(nodes.len(), 2);
    assert!(nodes.contains(&node_1));
    assert!(nodes.contains(&node_2));

    let expected_node_1 = store
        .cluster_table
        .get(&node_1.public_key)
        .unwrap()
        .unwrap();
    assert_eq!(expected_node_1, node_1);

    let expected_node_2 = store
        .cluster_table
        .get(&node_2.public_key)
        .unwrap()
        .unwrap();
    assert_eq!(expected_node_2, node_2);

    let mut ws = storage::WriteSet::default();
    store
        .cluster_table
        .remove_node(&node_1.public_key, &mut ws)
        .unwrap();
    store.commit(ws).unwrap();
    assert_eq!(store.cluster_table.nodes().unwrap().len(), 1);

    let mut ws = storage::WriteSet::default();
    store
        .cluster_table
        .remove_node(&node_2.public_key, &mut ws)
        .unwrap();
    store.commit(ws).unwrap();
    assert!(store.cluster_table.is_empty().unwrap());
}

#[test]
pub fn test_add_nodes_with_duplicate() {
    let (_, store) = common::test_storage();
    let mut ws = storage::WriteSet::default();
    let node_1 = Peer {
        address: Some("/ip4/192.168.0.1".parse().unwrap()),
        public_key: PrivateKey::generate().public_key(),
    };
    store.cluster_table.add_node(&node_1, &mut ws).unwrap();
    store.commit(ws).unwrap();

    let node_2 = Peer {
        address: Some("/ip4/192.168.0.1".parse().unwrap()),
        public_key: PrivateKey::generate().public_key(),
    };

    let mut ws = storage::WriteSet::default();
    assert!(store.cluster_table.add_node(&node_2, &mut ws).is_err());

    assert!(store
        .cluster_table
        .add_node(
            &Peer {
                address: Some("/ip4/192.168.0.3".parse().unwrap()),
                public_key: node_1.public_key
            },
            &mut ws
        )
        .is_err());
}
