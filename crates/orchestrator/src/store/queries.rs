use std::sync::Arc;
use storage::{EveStorage, WriteSet};
use types::ai::{
    query::{Query, QueryId},
    request::SignedAiRequest,
};

#[derive(Clone)]
pub struct Queries {
    storage: Arc<EveStorage>,
}

impl Queries {
    pub fn new(storage: Arc<EveStorage>) -> Self {
        Self { storage }
    }

    pub fn new_query(
        &self,
        id: QueryId,
        request: SignedAiRequest,
    ) -> Result<Query, storage::StorageError> {
        let mut ws = WriteSet::default();
        let user_seq = self
            .storage
            .sequence_table
            .increment_and_get(&request.query.pubkey, &mut ws)?;

        let query = Query::new(id, user_seq, request);
        self.storage.query_table.put_query(&query, &mut ws)?;
        self.storage.commit(ws)?;
        Ok(query)
    }

    pub(crate) fn update_query(&self, query: &Query) -> Result<(), storage::StorageError> {
        let mut ws = WriteSet::default();
        self.storage.query_table.put_query(query, &mut ws)?;
        self.storage.commit(ws)
    }
}
