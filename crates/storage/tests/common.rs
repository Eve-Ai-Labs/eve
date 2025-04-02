use storage::EveStorage;
use tempdir::TempDir;

pub fn test_storage() -> (TempDir, EveStorage) {
    let tmp_dir = TempDir::new("rocksdb").unwrap();
    let path = tmp_dir.path();
    let store = EveStorage::new(path, &Default::default()).unwrap();
    (tmp_dir, store)
}
