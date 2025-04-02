use super::{db::EveDB, iter::TableIter, tx::WriteSet, KEY_OPTIONS, VALUE_OPTIONS};
use crate::core::error::StorageError;
use bincode::Options as _;
use eyre::Result;
use node_config::db::RocksdbConfig;
use rocksdb::{BlockBasedOptions, Cache, ColumnFamilyDescriptor, DBCompressionType, Options};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

pub struct Table<K, V> {
    db: Arc<EveDB>,
    cf: &'static str,
    _marker: std::marker::PhantomData<(K, V)>,
}

impl<K, V> Table<K, V> {
    pub fn new(db: Arc<EveDB>, cf: &'static str) -> Result<Self> {
        Ok(Self {
            db,
            cf,
            _marker: std::marker::PhantomData,
        })
    }
}

impl<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> Table<K, V> {
    pub fn get(&self, key: &K) -> Result<Option<V>, StorageError> {
        let key = KEY_OPTIONS.serialize(key)?;

        let value = self.db.get(self.db.get_cf_handle(self.cf)?, &key)?;
        match value {
            Some(value) => Ok(Some(VALUE_OPTIONS.deserialize(&value)?)),
            None => Ok(None),
        }
    }

    pub fn iter<'a>(&'a self, from: Option<&K>) -> Result<TableIter<'a, K, V>, StorageError> {
        let cf = self.db.get_cf_handle(self.cf)?;

        let from = if let Some(from) = from {
            Some(KEY_OPTIONS.serialize(from)?)
        } else {
            None
        };
        let iter = self.db.iter(cf, from.as_deref())?;
        Ok(TableIter::new(iter))
    }

    pub fn scan<'a, P: Serialize>(
        &'a self,
        prefix: &P,
    ) -> Result<TableIter<'a, K, V>, StorageError> {
        let cf = self.db.get_cf_handle(self.cf)?;
        let from = KEY_OPTIONS.serialize(prefix)?;
        let iter = self.db.prefix_iterator(cf, &from)?;
        Ok(TableIter::new(iter))
    }

    pub fn is_empty(&self) -> Result<bool, StorageError> {
        Ok(self.iter(None)?.next().is_none())
    }

    pub fn put(&self, key: &K, value: &V, batch: &mut WriteSet) -> Result<(), StorageError> {
        let key = KEY_OPTIONS.serialize(key)?;
        let value = VALUE_OPTIONS.serialize(value)?;
        batch
            .inner
            .put_cf(self.db.get_cf_handle(self.cf)?, &key, &value);
        batch.has_diff = true;
        Ok(())
    }

    pub fn contains(&self, key: &K) -> Result<bool, StorageError> {
        let key = KEY_OPTIONS.serialize(key)?;
        let value = self.db.get(self.db.get_cf_handle(self.cf)?, &key)?;
        Ok(value.is_some())
    }

    pub fn delete(&self, key: &K, batch: &mut WriteSet) -> Result<(), StorageError> {
        let key = KEY_OPTIONS.serialize(key)?;
        batch.inner.delete_cf(self.db.get_cf_handle(self.cf)?, &key);
        batch.has_diff = true;
        Ok(())
    }
}

pub fn family_descriptor(
    cf_name: &str,
    cfg: &RocksdbConfig,
    prefix_len: Option<usize>,
) -> ColumnFamilyDescriptor {
    let mut table_options = BlockBasedOptions::default();

    if let Some(cache_index_and_filter_blocks) = cfg.cache_index_and_filter_blocks {
        table_options.set_cache_index_and_filter_blocks(cache_index_and_filter_blocks);
    }

    if let Some(block_size) = cfg.block_size {
        table_options.set_block_size(block_size as usize);
    }

    if let Some(block_cache_size) = cfg.block_cache_size {
        let cache = Cache::new_lru_cache(block_cache_size as usize);
        table_options.set_block_cache(&cache);
    }

    let mut cf_opts = Options::default();
    cf_opts.set_compression_type(DBCompressionType::Lz4);
    cf_opts.set_block_based_table_factory(&table_options);
    if let Some(prefix_len) = prefix_len {
        cf_opts.set_prefix_extractor(rocksdb::SliceTransform::create_fixed_prefix(prefix_len));
    }
    ColumnFamilyDescriptor::new((*cf_name).to_string(), cf_opts)
}
