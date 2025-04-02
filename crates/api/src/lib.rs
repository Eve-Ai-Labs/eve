pub mod account;
pub mod ai_models;
pub mod answer;
pub mod cluster;
pub mod history;
pub mod jwt_auth;
pub mod middleware;
pub mod nodes;
pub mod query;
pub mod status;

use crate::middleware::limits::LimitsMap;
use cluster::Cluster;
use metrics::Metrics;
use node_config::api::ApiConfig;
use orchestrator::ApiSender;
use poem::{
    get, handler,
    listener::TcpListener,
    middleware::{AddData, AddDataEndpoint, Cors, Tracing, TracingEndpoint},
    post,
    web::{Data, Json},
    EndpointExt, Route, Server,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use storage::EveStorage;
use tokio::task::JoinHandle;
use tracing::error;
use types::cluster::MetricsInfo;

struct AppState {
    sender: ApiSender,
    storage: Arc<EveStorage>,
    ai_limits: LimitsMap,
    airdrop_limits: LimitsMap,
    cfg: Arc<ApiConfig>,
    cluster: Cluster,
    metrics: Metrics,
}

fn route(state: AppState) -> AddDataEndpoint<TracingEndpoint<Route>, Arc<AppState>> {
    Route::new()
        .at("/", get(status::handler_status))
        .at("/ai", get(ai_models::handler_ai_model))
        .at("/query", post(query::handler_query))
        .at("/answer/:query_id", get(answer::handler_answer))
        .at("/history/:query_id", get(history::handler_history))
        .nest("/account", account::route())
        .at("/info", get(status::handler_info))
        .nest("/nodes", nodes::route(state.cfg.jwt))
        .nest("/metrics", get(handler_metrics))
        .with(Tracing)
        .with(AddData::new(Arc::new(state)))
}

#[handler]
async fn handler_metrics(state: Data<&Arc<AppState>>) -> poem::Result<Json<MetricsInfo>> {
    let mut info = state.metrics.metrics();

    Ok(Json(info))
}

pub async fn run(
    listen: SocketAddr,
    sender: ApiSender,
    storage: Arc<EveStorage>,
    cfg: ApiConfig,
) -> JoinHandle<()> {
    let cfg = Arc::new(cfg);
    let metrics = Metrics::default();
    let state = AppState {
        sender: sender.clone(),
        storage,
        ai_limits: LimitsMap::new(cfg.req_per_hour),
        airdrop_limits: LimitsMap::new(cfg.airdrop_per_hour),
        cfg: Arc::clone(&cfg),
        cluster: Cluster::new(sender, Duration::from_secs(cfg.cluster_info_ttl_secs)),
        metrics,
    };
    let server = Server::new(TcpListener::bind(listen)).run(route(state).with(Cors::new()));
    tokio::spawn(async move {
        if let Err(err) = server.await {
            error!("API Server error: {err}");
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::{cluster::Cluster, route, LimitsMap};
    use crypto::ed25519::private::PrivateKey;
    use node_config::api::ApiConfig;
    use orchestrator::mock::OrchestratorMock;
    use poem::{
        http::{header::RETRY_AFTER, StatusCode},
        test::TestClient,
    };
    use std::{sync::Arc, time::Duration};
    use storage::EveStorage;
    use tempfile::tempdir;
    use tracing::info;
    use tracing_test::traced_test;
    use types::ai::{
        query::{Query, QueryId},
        request::AiRequest,
    };

    #[tokio::test]
    #[traced_test]
    async fn test_info() {
        let tmp = tempdir().unwrap();
        let db_path = tmp.path().join("test.db");
        let db_config = Default::default();

        let eve = Arc::new(EveStorage::new(&db_path, &db_config).unwrap());
        let (sender, _) = tokio::sync::mpsc::channel(100);
        let cfg: Arc<ApiConfig> = Default::default();

        let client = TestClient::new(route(crate::AppState {
            storage: eve.clone(),
            sender: sender.clone(),
            ai_limits: LimitsMap::new(cfg.req_per_hour),
            airdrop_limits: LimitsMap::new(cfg.airdrop_per_hour),
            cfg: Arc::clone(&cfg),
            cluster: Cluster::new(sender, Duration::from_secs(cfg.cluster_info_ttl_secs)),
            metrics: Default::default(),
        }));

        let response = client.get("/").send().await;
        response.assert_status_is_ok();
    }

    #[tokio::test]
    #[traced_test]
    async fn test_query() {
        let tmp = tempdir().unwrap();
        let db_path = tmp.path().join("test.db");
        let db_config = Default::default();
        let eve = Arc::new(EveStorage::new(&db_path, &db_config).unwrap());

        let user_private_key = PrivateKey::generate();
        let user_pubkey = user_private_key.public_key();

        let (sender, _handle) = OrchestratorMock::create(eve.clone());
        let cfg: Arc<ApiConfig> = Default::default();

        let client = TestClient::new(route(crate::AppState {
            storage: eve.clone(),
            sender: sender.clone(),
            ai_limits: LimitsMap::new(cfg.req_per_hour),
            airdrop_limits: LimitsMap::new(cfg.airdrop_per_hour),
            cfg: Arc::clone(&cfg),
            cluster: Cluster::new(sender, Duration::from_secs(cfg.cluster_info_ttl_secs)),
            metrics: Default::default(),
        }));

        let response = client
            .post("/query")
            .body_json(
                &AiRequest::new("test".into(), vec![], user_pubkey)
                    .sign(&user_private_key)
                    .unwrap(),
            )
            .send()
            .await;
        info!("{:#?}", response.0);
        response.assert_status_is_ok();

        let query_id: QueryId = response.0.into_body().into_json().await.unwrap();
        info!("{query_id:?}",);

        let response = client.get(format!("/answer/{query_id}")).send().await;
        info!("{:#?}", response.0);
        response.assert_status_is_ok();

        let query: Query = response.0.into_body().into_json().await.unwrap();
        info!("{query:#?}",);
    }

    #[tokio::test]
    #[traced_test]
    async fn test_restrictions() {
        let tmp = tempdir().unwrap();
        let db_path = tmp.path().join("test.db");
        let db_config = Default::default();
        let eve = Arc::new(EveStorage::new(&db_path, &db_config).unwrap());

        let user_private_key = PrivateKey::generate();
        let user_pubkey = user_private_key.public_key();

        let (sender, _handle) = OrchestratorMock::create(eve.clone());
        let cfg = Arc::new(ApiConfig {
            blacklist_words: vec!["bad".to_string()],
            req_per_hour: 10,
            max_req_length: 6,
            ..Default::default()
        });

        let client = TestClient::new(route(crate::AppState {
            storage: eve.clone(),
            sender: sender.clone(),
            ai_limits: LimitsMap::new(cfg.req_per_hour),
            airdrop_limits: LimitsMap::new(cfg.airdrop_per_hour),
            cfg: Arc::clone(&cfg),
            cluster: Cluster::new(sender, Duration::from_secs(cfg.cluster_info_ttl_secs)),
            metrics: Default::default(),
        }));
        let req = AiRequest::new("test".into(), vec![], user_pubkey)
            .sign(&user_private_key)
            .unwrap();

        let response = client.post("/query").body_json(&req.clone()).send().await;
        response.assert_status_is_ok();

        let response = client
            .post("/query")
            .body_json(
                &AiRequest::new("1234567".into(), vec![], user_pubkey)
                    .sign(&user_private_key)
                    .unwrap(),
            )
            .send()
            .await;
        response.assert_status(StatusCode::PAYLOAD_TOO_LARGE);

        let response = client
            .post("/query")
            .body_json(
                &AiRequest::new("my bad".into(), vec![], user_pubkey)
                    .sign(&user_private_key)
                    .unwrap(),
            )
            .send()
            .await;
        response.assert_status(StatusCode::BAD_REQUEST);

        for _ in 0..9 {
            let response = client.post("/query").body_json(&req.clone()).send().await;
            response.assert_status_is_ok();
        }

        let response = client.post("/query").body_json(&req.clone()).send().await;
        response.assert_status(StatusCode::TOO_MANY_REQUESTS);
        response.assert_header_exist(RETRY_AFTER);
        let resp = response.0;
        let header = resp.header(RETRY_AFTER);
        info!(?header);

        let user_private_key = PrivateKey::generate();
        let user_pubkey = user_private_key.public_key();

        let response = client
            .post("/query")
            .body_json(
                &AiRequest::new("test".into(), vec![], user_pubkey)
                    .sign(&user_private_key)
                    .unwrap(),
            )
            .send()
            .await;
        response.assert_status_is_ok();
    }
}
