use dirtybase_db::macros::DirtyTable;

use crate::app::cache_manager::cache_entry::CacheEntry;

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(table = "core_cache", id = "key")]
pub struct CacheDbStoreEntity {
    pub(crate) key: String,
    pub(crate) value: String,
    pub(crate) expiration: Option<i64>,
}

impl From<CacheEntry> for CacheDbStoreEntity {
    fn from(value: CacheEntry) -> Self {
        Self {
            key: value.key,
            value: value.value,
            expiration: value.expiration,
        }
    }
}

impl From<CacheDbStoreEntity> for CacheEntry {
    fn from(value: CacheDbStoreEntity) -> Self {
        Self::new(&value.key, &value.value, value.expiration)
    }
}

impl From<&CacheDbStoreEntity> for CacheEntry {
    fn from(value: &CacheDbStoreEntity) -> Self {
        Self::new(&value.key, &value.value, value.expiration)
    }
}
