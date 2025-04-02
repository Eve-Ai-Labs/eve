use super::tx::WriteSet;
use crate::StorageError;
use eyre::Result;
use node_config::db::RocksdbConfig;
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBCompressionType, DBIteratorWithThreadMode,
    DBWithThreadMode, Options, SingleThreaded,
};
use std::{collections::HashSet, path::Path};

#[derive(Debug)]
pub struct EveDB {
    inner: rocksdb::DB,
}

impl EveDB {
    pub fn open_cf<P: AsRef<Path>>(
        db_opts: &Options,
        path: P,
        cfds: Vec<ColumnFamilyDescriptor>,
    ) -> Result<EveDB> {
        let existing_cfs = rocksdb::DB::list_cf(db_opts, path.as_ref()).unwrap_or_default();

        let unrecognized_cfds = existing_cfs
            .iter()
            .map(AsRef::as_ref)
            .collect::<HashSet<&str>>()
            .difference(&cfds.iter().map(|cfd| cfd.name()).collect())
            .map(|cf| {
                let mut cf_opts = Options::default();
                cf_opts.set_compression_type(DBCompressionType::Lz4);
                ColumnFamilyDescriptor::new(cf.to_string(), cf_opts)
            })
            .collect::<Vec<_>>();
        let all_cfds = cfds.into_iter().chain(unrecognized_cfds);

        let inner = rocksdb::DB::open_cf_descriptors(db_opts, path.as_ref(), all_cfds)?;
        Ok(EveDB { inner })
    }

    pub fn get(&self, cf: &ColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(self.inner.get_cf(cf, key)?)
    }

    pub fn commit(&self, batch: WriteSet) -> Result<(), StorageError> {
        if batch.has_diff {
            self.inner.write_opt(batch.inner, &options())?;
        }
        Ok(())
    }

    pub fn iter<'a>(
        &'a self,
        cf: &ColumnFamily,
        from: Option<&[u8]>,
    ) -> Result<DBIteratorWithThreadMode<'a, DBWithThreadMode<SingleThreaded>>, StorageError> {
        let raw_iter = if let Some(from) = from {
            self.inner.iterator_cf(
                cf,
                rocksdb::IteratorMode::From(from, rocksdb::Direction::Forward),
            )
        } else {
            self.inner.iterator_cf(cf, rocksdb::IteratorMode::Start)
        };

        Ok(raw_iter)
    }

    pub fn prefix_iterator<'a>(
        &'a self,
        cf: &ColumnFamily,
        prefix: &[u8],
    ) -> Result<DBIteratorWithThreadMode<'a, DBWithThreadMode<SingleThreaded>>, StorageError> {
        Ok(self.inner.prefix_iterator_cf(cf, prefix))
    }

    pub fn get_cf_handle(
        &self,
        cf_name: &'static str,
    ) -> Result<&rocksdb::ColumnFamily, StorageError> {
        self.inner
            .cf_handle(cf_name)
            .ok_or(StorageError::ColumnFamilyNotFound(cf_name))
    }
}

pub fn make_options(cfg: &RocksdbConfig) -> Options {
    let mut db_opts = Options::default();

    if let Some(max_open_files) = cfg.max_open_files {
        db_opts.set_max_open_files(max_open_files);
    }
    if let Some(max_background_jobs) = cfg.max_background_jobs {
        db_opts.set_max_background_jobs(max_background_jobs);
    }

    if let Some(max_total_wal_size) = cfg.max_total_wal_size {
        db_opts.set_max_total_wal_size(max_total_wal_size);
    }

    db_opts.create_if_missing(true);
    db_opts.create_missing_column_families(true);

    db_opts
}

fn options() -> rocksdb::WriteOptions {
    let mut opts = rocksdb::WriteOptions::default();
    opts.set_sync(true);
    opts
}
