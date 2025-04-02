use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DbConfig {
    pub path: PathBuf,
    pub rocksdb: RocksdbConfig,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            path: "db".into(),
            rocksdb: RocksdbConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RocksdbConfig {
    /// Maximum number of files open by RocksDB at one time
    pub max_open_files: Option<i32>,
    /// Maximum size of the RocksDB write ahead log (WAL)
    pub max_total_wal_size: Option<u64>,
    /// Maximum number of background threads for Rocks DB
    pub max_background_jobs: Option<i32>,
    /// Block cache size for Rocks DB
    pub block_cache_size: Option<u64>,
    /// Block size for Rocks DB
    pub block_size: Option<u64>,
    /// Whether cache index and filter blocks into block cache.
    pub cache_index_and_filter_blocks: Option<bool>,
}

impl Default for RocksdbConfig {
    fn default() -> Self {
        Self {
            max_open_files: Some(5000),
            max_total_wal_size: Some(1u64 << 30),
            max_background_jobs: Some(16),
            block_cache_size: Some(8 * (1u64 << 20)),
            block_size: Some(4 * (1u64 << 10)),
            cache_index_and_filter_blocks: Some(false),
        }
    }
}
