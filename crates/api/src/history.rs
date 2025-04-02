use crate::AppState;
use poem::{
    handler,
    web::{Data, Json, Path},
};
use std::sync::Arc;
use tracing::debug;
use types::ai::{query::QueryId, request::History};

#[handler]
pub(crate) async fn handler_history(
    Path(query_id): Path<QueryId>,
    state: Data<&Arc<AppState>>,
) -> poem::Result<Json<Vec<History>>> {
    debug!("query_id: {query_id}");

    let Some(query) = state.storage.query_table.get_query(&query_id)? else {
        return Ok(Json(Default::default()));
    };

    Ok(Json(query.as_history()))
}
