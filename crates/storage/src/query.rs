use crate::core::{error::StorageError, table::Table, tx::WriteSet};
use crypto::ed25519::public::PublicKey;
use eyre::Result;
use types::ai::query::{Query, QueryId};

pub const QUERY_TABLE_NAME: &str = "query-table";
pub const QUERY_IN_PROGRESS: &str = "query-in-progress";
pub const QUERY_BY_PUB_KEY: &str = "query-by-public-key";

pub type PubkeyIndexKey = (PublicKey, u64);
pub type PubkeyIndex = Table<PubkeyIndexKey, QueryId>;

pub struct QueryTable {
    table: Table<QueryId, Query>,
    in_progress: Table<QueryId, QueryId>,
    by_public_key: PubkeyIndex,
}

impl QueryTable {
    pub fn new(
        table: Table<QueryId, Query>,
        in_progress: Table<QueryId, QueryId>,
        by_public_key: PubkeyIndex,
    ) -> Self {
        Self {
            table,
            in_progress,
            by_public_key,
        }
    }

    pub fn put_query(&self, query: &Query, ws: &mut WriteSet) -> Result<(), StorageError> {
        self.table.put(&query.id, query, ws)?;
        if query.is_complete() {
            self.in_progress.delete(&query.id, ws)?;
        } else {
            self.in_progress.put(&query.id, &query.id, ws)?;
        }
        self.by_public_key
            .put(&(query.request.query.pubkey, query.sequence), &query.id, ws)?;
        Ok(())
    }

    pub fn get_query(&self, query_id: &QueryId) -> Result<Option<Query>, StorageError> {
        self.table.get(query_id)
    }

    pub fn users_query_ids(
        &self,
        pubkey: &PublicKey,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<QueryId>, StorageError> {
        let mut query_ids = Vec::with_capacity(limit);
        let iter = self.by_public_key.scan(pubkey)?.skip(offset).take(limit);

        for query_id in iter {
            let entry = query_id?;
            query_ids.push(entry.1);
        }
        Ok(query_ids)
    }

    pub fn get_in_progress_ids(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<QueryId>, StorageError> {
        let mut query_ids = Vec::with_capacity(limit);
        let iter = self.in_progress.iter(None)?.skip(offset);

        for query_id in iter {
            let id = query_id?;
            query_ids.push(id.1);
            if query_ids.len() >= limit {
                break;
            }
        }
        Ok(query_ids)
    }
}
