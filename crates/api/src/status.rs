use crate::AppState;
use poem::{
    handler,
    web::{Data, Json},
    IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;
use types::cluster::ClusterInfo;

/// The start page. Status Service
#[handler]
pub fn handler_status() -> ApiStatus {
    ApiStatus { cost: 1 }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiStatus {
    cost: u64,
}

impl IntoResponse for ApiStatus {
    fn into_response(self) -> poem::Response {
        Json(self).into_response()
    }
}

#[handler]
pub async fn handler_info(state: Data<&Arc<AppState>>) -> poem::Result<Json<ClusterInfo>> {
    debug!("Get cluster info");
    let mut info = state.cluster.load_info().await?;
    Ok(Json(info.cluster_info))
}
