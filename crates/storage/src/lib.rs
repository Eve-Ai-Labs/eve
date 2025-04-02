pub mod account;
pub mod cluster;
mod core;
pub mod query;
pub mod sequence;

use account::ACCOUNT_TABLE_NAME;
use cluster::{CLUSTER_ADDRESS_TABLE_NAME, CLUSTER_TABLE_NAME};
use core::{
    db::{make_options, EveDB},
    table::{family_descriptor, Table},
};
pub use core::{error::StorageError, tx::WriteSet};
use eyre::Result;
use node_config::db::RocksdbConfig;
use query::{QUERY_BY_PUB_KEY, QUERY_IN_PROGRESS, QUERY_TABLE_NAME};
use sequence::SEQUENCE_TABLE_NAME;
use std::{path::Path, sync::Arc};

pub struct EveStorage {
    db: Arc<EveDB>,
    pub query_table: query::QueryTable,
    pub sequence_table: sequence::SequenceTable,
    pub cluster_table: cluster::ClusterTable,
    pub account_table: account::AccountsTable,
}

impl EveStorage {
    pub fn new<P: AsRef<Path>>(db_path: P, cfg: &RocksdbConfig) -> Result<Self> {
        let db = Arc::new(EveDB::open_cf(
            &make_options(cfg),
            &db_path,
            vec![
                family_descriptor(QUERY_TABLE_NAME, cfg, None),
                family_descriptor(QUERY_IN_PROGRESS, cfg, None),
                family_descriptor(QUERY_BY_PUB_KEY, cfg, Some(32)),
                family_descriptor(SEQUENCE_TABLE_NAME, cfg, None),
                family_descriptor(CLUSTER_TABLE_NAME, cfg, None),
                family_descriptor(CLUSTER_ADDRESS_TABLE_NAME, cfg, None),
                family_descriptor(ACCOUNT_TABLE_NAME, cfg, None),
            ],
        )?);

        let query_table = query::QueryTable::new(
            Table::new(db.clone(), QUERY_TABLE_NAME)?,
            Table::new(db.clone(), QUERY_IN_PROGRESS)?,
            Table::new(db.clone(), QUERY_BY_PUB_KEY)?,
        );

        let sequence_table =
            sequence::SequenceTable::new(Table::new(db.clone(), SEQUENCE_TABLE_NAME)?);

        let cluster_table = cluster::ClusterTable::new(
            Table::new(db.clone(), CLUSTER_TABLE_NAME)?,
            Table::new(db.clone(), CLUSTER_ADDRESS_TABLE_NAME)?,
        );

        let account_table =
            account::AccountsTable::new(Table::new(db.clone(), ACCOUNT_TABLE_NAME)?);

        Ok(Self {
            db,
            query_table,
            sequence_table,
            cluster_table,
            account_table,
        })
    }

    pub fn commit(&self, ws: WriteSet) -> Result<(), StorageError> {
        self.db.commit(ws)?;
        Ok(())
    }
}
