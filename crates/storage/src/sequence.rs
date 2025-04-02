use crate::core::{error::StorageError, table::Table, tx::WriteSet};
use crypto::ed25519::public::PublicKey;

pub const SEQUENCE_TABLE_NAME: &str = "sequence";

pub struct SequenceTable {
    table: Table<PublicKey, u64>,
}

impl SequenceTable {
    pub fn new(table: Table<PublicKey, u64>) -> Self {
        Self { table }
    }

    pub fn increment_and_get(
        &self,
        key: &PublicKey,
        ws: &mut WriteSet,
    ) -> Result<u64, StorageError> {
        let current = self.table.get(key)?.unwrap_or(0);
        self.table.put(key, &(current + 1), ws)?;
        Ok(current + 1)
    }

    pub fn get(&self, key: &PublicKey) -> Result<u64, StorageError> {
        Ok(self.table.get(key)?.unwrap_or(0))
    }
}
