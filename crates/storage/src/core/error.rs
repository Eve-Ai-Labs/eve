use thiserror::Error;
use types::ai::query::QueryId;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("rocksdb error: {0}")]
    RocksDb(#[from] rocksdb::Error),
    #[error("Error while serializing query: {0}")]
    Serde(#[from] bincode::Error),
    #[error("DB::cf_handle not found for column family name:{0}")]
    ColumnFamilyNotFound(&'static str),
    #[error("Query not found: {0}")]
    QueryNotFound(QueryId),
    #[error("Corrupted data")]
    CorruptedData,
    #[error("Already exists")]
    AlreadyExists,
}

#[cfg(feature = "err_poem")]
use poem::{error::ResponseError, http::StatusCode};

#[cfg(feature = "err_poem")]
impl ResponseError for StorageError {
    fn status(&self) -> StatusCode {
        match self {
            StorageError::RocksDb(_)
            | StorageError::Serde(_)
            | StorageError::ColumnFamilyNotFound(_)
            | StorageError::CorruptedData => StatusCode::INTERNAL_SERVER_ERROR,
            StorageError::QueryNotFound(_) => StatusCode::BAD_REQUEST,
            StorageError::AlreadyExists => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
