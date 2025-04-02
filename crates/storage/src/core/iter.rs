use super::{KEY_OPTIONS, VALUE_OPTIONS};
use crate::core::error::StorageError;
use bincode::Options;
use rocksdb::{DBIteratorWithThreadMode, DBWithThreadMode, SingleThreaded};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

pub struct TableIter<'a, K, V> {
    iter: DBIteratorWithThreadMode<'a, DBWithThreadMode<SingleThreaded>>,
    phantom: PhantomData<(K, V)>,
}

impl<'a, K, V> TableIter<'a, K, V>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    pub(super) fn new(
        iter: DBIteratorWithThreadMode<'a, DBWithThreadMode<SingleThreaded>>,
    ) -> Self {
        Self {
            iter,
            phantom: PhantomData,
        }
    }
}

impl<K, V> Iterator for TableIter<'_, K, V>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    type Item = Result<(K, V), StorageError>;

    fn next(&mut self) -> Option<Self::Item> {
        let key_value = self.iter.next()?;

        match key_value {
            Ok((key, value)) => {
                let key = KEY_OPTIONS
                    .deserialize::<K>(&key)
                    .map_err(StorageError::Serde);
                let value = VALUE_OPTIONS
                    .deserialize::<V>(&value)
                    .map_err(StorageError::Serde);
                match (key, value) {
                    (Ok(key), Ok(value)) => Some(Ok((key, value))),
                    (Err(err), _) | (_, Err(err)) => Some(Err(err)),
                }
            }
            Err(err) => Some(Err(StorageError::RocksDb(err))),
        }
    }
}
