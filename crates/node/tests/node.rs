use crypto::hash::sha3;
use futures::StreamExt;
use p2p::{etp::ToETP, key::ToP2P as _};
use types::{ai::request::AiRequest, p2p::NodeMessage};

mod rt;

#[tokio::test]
pub async fn test_node_response() {
    let mut node = rt::start_node().await;

    for i in 0..50 {
        let id = sha3(&i);
        let req = AiRequest::new(format!("test:{}", i), vec![], node.orch.public_key());
        node.send(id, req.sign(&node.orch).unwrap()).await;
    }

    let mut i = 0;
    while i < 50 {
        let expected_id = sha3(&i);
        let resp = node.from_node.next().await.unwrap();
        match resp {
            ToETP::Send { to, message, .. } => {
                assert_eq!(to, node.orch.public_key().to_p2p().to_peer_id());
                match message {
                    types::p2p::EveMessage::Orch(orch_message) => {
                        panic!("unexpected message: {:?}", orch_message)
                    }
                    types::p2p::EveMessage::Node(node_message) => match node_message {
                        NodeMessage::AiResponse { id, response } => {
                            assert_eq!(expected_id, id);
                            let response = response
                                .unwrap()
                                .verify()
                                .unwrap()
                                .into_inner()
                                .node_response;
                            assert_eq!(response.response, format!("ai:test:{}", i));
                            assert_eq!(response.pubkey, node.node_key.public_key());
                        }
                    },
                }
                i += 1;
            }
            ToETP::Dial(_, _) => {}
            _ => panic!("unexpected message: {:?}", resp),
        }
    }
}
