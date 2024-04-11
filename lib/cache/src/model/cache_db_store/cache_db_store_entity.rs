use dirtybase_db_macro::DirtyTable;

use crate::cache_manager::cache_entry::CacheEntry;

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(table = "core_cache", id = "key")]
pub struct CacheDbStoreEntity {
    pub(crate) key: String,
    pub(crate) content: String,
    pub(crate) expiration: Option<i64>,
}

impl From<CacheEntry> for CacheDbStoreEntity {
    fn from(value: CacheEntry) -> Self {
        Self {
            key: value.key,
            content: value.value,
            expiration: value.expiration,
        }
    }
}

impl From<CacheDbStoreEntity> for CacheEntry {
    fn from(value: CacheDbStoreEntity) -> Self {
        Self::new(&value.key, &value.content, value.expiration)
    }
}

impl From<&CacheDbStoreEntity> for CacheEntry {
    fn from(value: &CacheDbStoreEntity) -> Self {
        Self::new(&value.key, &value.content, value.expiration)
    }
}
