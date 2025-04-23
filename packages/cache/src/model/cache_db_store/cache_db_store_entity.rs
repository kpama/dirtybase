use dirtybase_db_macro::DirtyTable;

use crate::cache_manager::cache_entry::CacheEntry;

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(table = "cache", id = "key")]
pub struct CacheDbStoreEntity {
    #[dirty(flatten)]
    inner: CacheEntry,
}

impl From<CacheEntry> for CacheDbStoreEntity {
    fn from(value: CacheEntry) -> Self {
        Self { inner: value }
    }
}

impl From<CacheDbStoreEntity> for CacheEntry {
    fn from(value: CacheDbStoreEntity) -> Self {
        value.inner
    }
}

impl From<&CacheDbStoreEntity> for CacheEntry {
    fn from(value: &CacheDbStoreEntity) -> Self {
        value.inner.clone()
    }
}
