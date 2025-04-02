use crate::AppState;
use crypto::ed25519::public::PublicKey;
use poem::{
    get, handler,
    http::StatusCode,
    post,
    web::{Data, Json, Path, RemoteAddr},
    IntoResponse, Route,
};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};
use tracing::debug;

pub fn route() -> Route {
    Route::new()
        .at("/:pubkey", get(handler_account))
        .at("/airdrop/:pubkey", post(handler_airdrop))
}

#[handler]
pub fn handler_account(
    state: Data<&Arc<AppState>>,
    Path(pubkey): Path<String>,
) -> poem::Result<AccountInfo> {
    debug!("address: {pubkey}");

    let account = state
        .storage
        .account_table
        .get(&PublicKey::from_str(&pubkey)?)?;

    Ok(if let Some(account) = account {
        AccountInfo {
            balance: account.balance,
        }
    } else {
        AccountInfo { balance: 0 }
    })
}

#[handler]
pub async fn handler_airdrop(
    remote_addr: &RemoteAddr,
    state: Data<&Arc<AppState>>,
    Path(pubkey): Path<String>,
) -> poem::Result<Json<AccountInfo>> {
    debug!("airdrop: {pubkey}");
    let public_key = PublicKey::from_str(&pubkey)?;

    state.airdrop_limits.pubkey_check(&public_key)?;
    if let Some(addr) = remote_addr.as_socket_addr() {
        state.ai_limits.ip_check(&addr.ip())?;
    }

    let airdrop_sum = 1_000_000;
    let (tx, rx) = tokio::sync::oneshot::channel();
    state
        .sender
        .send(orchestrator::OrchRequest::Airdrop {
            address: public_key,
            amount: airdrop_sum,
            tx,
        })
        .await
        .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;
    let _ = rx
        .await
        .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    let account = state.storage.account_table.get(&public_key)?;
    Ok(Json(AccountInfo {
        balance: account.map_or(0, |account| account.balance),
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    balance: u64,
}

impl IntoResponse for AccountInfo {
    fn into_response(self) -> poem::Response {
        Json(self).into_response()
    }
}
