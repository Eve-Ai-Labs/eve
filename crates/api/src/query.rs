use crate::AppState;
use crypto::hash::Hash;
use eyre::eyre;
use orchestrator::OrchRequest;
use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json, RemoteAddr},
};
use std::sync::Arc;
use types::ai::request::SignedAiRequest;

#[handler]
pub async fn handler_query(
    remote_addr: &RemoteAddr,
    Json(request): Json<SignedAiRequest>,
    state: Data<&Arc<AppState>>,
) -> poem::Result<Json<Hash>> {
    if request.query.message.len() > state.cfg.max_req_length {
        return Err(poem::Error::from_status(StatusCode::PAYLOAD_TOO_LARGE));
    }
    if state
        .cfg
        .blacklist_words
        .iter()
        .any(|word| request.query.message.contains(word))
    {
        return Err(poem::Error::from_status(StatusCode::BAD_REQUEST));
    }
    state.ai_limits.pubkey_check(&request.query.pubkey)?;
    if let Some(addr) = remote_addr.as_socket_addr() {
        state.ai_limits.ip_check(&addr.ip())?;
    }
    let verified_request = request.verify()?;
    let (sender_response, receiver_response) = tokio::sync::oneshot::channel();

    let orch_request = OrchRequest::Ask {
        request: verified_request,
        tx: sender_response,
    };

    state
        .sender
        .send(orch_request)
        .await
        .map_err(|err| eyre!("{err}"))?;

    let query_id = receiver_response.await.map_err(|err| eyre!("{err}"))??;
    Ok(Json(query_id))
}
