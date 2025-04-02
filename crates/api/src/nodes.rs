use crate::{jwt_auth::JwtAuth, AppState};
use crypto::ed25519::public::PublicKey;
use jwt::JwtSecret;
use p2p::key::ToP2P;
use poem::{
    get, handler,
    http::StatusCode,
    put,
    web::{Data, Json, Path},
    EndpointExt, Route,
};
use std::sync::Arc;
use tracing::{debug, error};
use types::{
    cluster::{Node, NodeInfo},
    p2p::Peer,
};

pub fn route(jwt: JwtSecret) -> Route {
    Route::new()
        .at("/", get(handler_list))
        .at(
            "/action",
            put(handler_put).delete(handler_delete).with(JwtAuth(jwt)),
        )
        .at("/:pubkey", get(handler_get))
}

#[handler]
async fn handler_list(state: Data<&Arc<AppState>>) -> poem::Result<Json<Vec<Node>>> {
    debug!("Get cluster nodes");
    let mut info = state.cluster.load_info().await?;
    Ok(Json(info.nodes.values().cloned().collect()))
}

#[handler]
async fn handler_get(
    Path(node_id): Path<PublicKey>,
    state: Data<&Arc<AppState>>,
) -> poem::Result<Json<Option<NodeInfo>>> {
    let mut info = state.cluster.load_info().await?;

    let node = info
        .nodes
        .get(&node_id.to_p2p().to_peer_id())
        .map(|node| NodeInfo {
            address: node.address.clone(),
            peer_id: node.peer_id,
            is_connected: node.is_connected(),
        });
    Ok(Json(node))
}

#[handler]
async fn handler_put(
    Json(node): Json<Peer>,
    state: Data<&Arc<AppState>>,
) -> poem::Result<Json<String>> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    state
        .sender
        .send(orchestrator::OrchRequest::AddNode {
            address: node.address,
            public_key: node.public_key,
            tx,
        })
        .await
        .map_err(|err| {
            debug!("Error: {err:?}");
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    rx.await
        .map_err(|err| {
            debug!("Error: {err:?}");
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })
        .map_err(|err| {
            error!("{err:?}");
            err
        })?
        .map_err(|err| {
            debug!("Error: {err:?}");
            poem::Error::from_string(format!("{err}"), StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    Ok(Json("Success".into()))
}

#[handler]
async fn handler_delete(
    Json(node): Json<PublicKey>,
    state: Data<&Arc<AppState>>,
) -> poem::Result<Json<String>> {
    debug!("Delete node: {node:#?}");

    let (tx, rx) = tokio::sync::oneshot::channel();
    state
        .sender
        .send(orchestrator::OrchRequest::RemoveNode {
            public_key: node,
            tx,
        })
        .await
        .map_err(|err| {
            debug!("Err: {err:?}");
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    rx.await
        .map_err(|err| {
            debug!("Error: {err:?}");
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?
        .map_err(|err| {
            debug!("Error: {err}");
            poem::Error::from_string(format!("{err}"), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
    Ok(Json("Success".into()))
}

#[cfg(test)]
mod tests {
    use crate::{cluster::Cluster, route, AppState, LimitsMap};
    use crypto::ed25519::private::PrivateKey;
    use node_config::api::ApiConfig;
    use orchestrator::{OrchRequest, OrchestratorError};
    use p2p::{key::ToP2P as _, task::PeerId};
    use poem::{
        http::{header::AUTHORIZATION, StatusCode},
        middleware::AddDataEndpoint,
        test::TestClient,
        Endpoint,
    };
    use std::{collections::HashMap, sync::Arc, time::Duration};
    use storage::EveStorage;
    use tempfile::tempdir;
    use tokio::{sync::mpsc::Receiver, task::JoinHandle};
    use tracing_test::traced_test;
    use types::{
        cluster::{ClusterInfo, ClusterInfoWithNodes, Node},
        p2p::Peer,
    };

    #[tokio::test]
    #[traced_test]
    async fn test_nodes() {
        let tmp = tempdir().unwrap();
        let db_path = tmp.path().join("test.db");
        let db_config = Default::default();

        let eve = Arc::new(EveStorage::new(&db_path, &db_config).unwrap());
        let (sender, orch_req) = tokio::sync::mpsc::channel(100);
        let hndl = orch_mock(orch_req);
        let cfg = Arc::new(ApiConfig {
            cluster_info_ttl_secs: 0,
            ..Default::default()
        });

        let jwt = cfg.jwt;
        let auth_head = jwt.to_bearer().unwrap();
        let client = TestClient::new(route(crate::AppState {
            storage: eve.clone(),
            sender: sender.clone(),
            ai_limits: LimitsMap::new(cfg.req_per_hour),
            airdrop_limits: LimitsMap::new(cfg.airdrop_per_hour),
            cfg: Arc::clone(&cfg),
            cluster: Cluster::new(sender, Duration::from_secs(cfg.cluster_info_ttl_secs)),
            metrics: Default::default(),
        }));

        let response = client.get("/nodes").send().await;
        response.assert_status_is_ok();
        assert!(response.0.content_type().unwrap().contains("json"));
        assert_eq!(0, get_nodes(&client).await);

        client
            .get("/nodes/action")
            .send()
            .await
            .assert_status(StatusCode::UNAUTHORIZED);
        client
            .put("/nodes/action")
            .send()
            .await
            .assert_status(StatusCode::UNAUTHORIZED);
        client
            .delete("/nodes/action")
            .send()
            .await
            .assert_status(StatusCode::UNAUTHORIZED);

        client
            .get("/nodes/action")
            .header(AUTHORIZATION, auth_head.clone())
            .send()
            .await
            .assert_status(StatusCode::METHOD_NOT_ALLOWED);

        let nodes_pub_keys = (0..5)
            .map(|_| PrivateKey::generate().public_key())
            .collect::<Vec<_>>();
        for (n, public_key) in nodes_pub_keys.iter().enumerate() {
            client
                .put("/nodes/action")
                .header(AUTHORIZATION, auth_head.clone())
                .body_json(&Peer {
                    address: None,
                    public_key: *public_key,
                })
                .send()
                .await
                .assert_status_is_ok();
            assert_eq!(n + 1, get_nodes(&client).await);
        }

        for (n, public_key) in nodes_pub_keys.iter().enumerate().rev() {
            client
                .delete("/nodes/action")
                .header(AUTHORIZATION, auth_head.clone())
                .body_json(&public_key)
                .send()
                .await
                .assert_status_is_ok();
            assert_eq!(n, get_nodes(&client).await);
        }

        hndl.abort();
    }

    async fn get_nodes<E>(client: &TestClient<AddDataEndpoint<E, Arc<AppState>>>) -> usize
    where
        E: Endpoint,
    {
        client
            .get("/nodes")
            .send()
            .await
            .json()
            .await
            .value()
            .array()
            .len()
    }

    fn orch_mock(mut rs: Receiver<OrchRequest>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut nodes: HashMap<PeerId, Node> = HashMap::default();
            let key = PrivateKey::generate().public_key();
            while let Some(req) = rs.recv().await {
                match req {
                    OrchRequest::Ask { request, tx: _ } => {
                        panic!("Unexpected request: {:?}", request)
                    }
                    OrchRequest::AddNode {
                        address,
                        public_key,
                        tx,
                    } => {
                        let has_peer = nodes.iter().any(|(_, node)| {
                            node.key == public_key
                                || (node.address.is_some() && node.address == address)
                        });
                        let result = if has_peer {
                            Err(OrchestratorError::NodeIsAlreadyInWhitelist(
                                public_key.to_p2p().to_peer_id(),
                            ))
                        } else {
                            let peer_id = public_key.to_p2p().to_peer_id();
                            nodes.insert(peer_id, Node::new(public_key, peer_id, address));
                            Ok(())
                        };

                        tx.send(result).unwrap();
                    }
                    OrchRequest::RemoveNode { public_key, tx } => {
                        if nodes.remove(&public_key.to_p2p().to_peer_id()).is_some() {
                            tx.send(Ok(())).unwrap();
                        } else {
                            tx.send(Err(OrchestratorError::NodeIsNotInWhitelist(
                                public_key.to_p2p().to_peer_id(),
                            )))
                            .unwrap();
                        }
                    }
                    OrchRequest::ClusterInfo { tx } => {
                        tx.send(ClusterInfoWithNodes {
                            cluster_info: ClusterInfo {
                                orch_address: vec![],
                                orch_pubkey: key,
                                webrtc_certhash: None,
                                nodes_count: nodes.len(),
                            },
                            nodes: nodes.clone(),
                        })
                        .unwrap();
                    }
                    OrchRequest::Airdrop { tx, .. } => {
                        tx.send(Ok(())).unwrap();
                    }
                }
            }
        })
    }
}
