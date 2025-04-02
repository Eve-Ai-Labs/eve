#[derive(Default)]
pub struct WriteSet {
    pub(super) inner: rocksdb::WriteBatch,
    pub(super) has_diff: bool,
}
