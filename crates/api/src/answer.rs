use crate::AppState;
use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json, Path},
};
use std::sync::Arc;
use types::ai::query::{Query, QueryId};

#[handler]
pub(crate) async fn handler_answer(
    Path(query_id): Path<QueryId>,
    state: Data<&Arc<AppState>>,
) -> poem::Result<Json<Query>> {
    let respose = state
        .storage
        .query_table
        .get_query(&query_id)?
        .ok_or(StatusCode::PROCESSING)?;

    Ok(Json(respose))
}
