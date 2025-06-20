use dirtybase_contract::db_contract::types::{InternalIdField, OptionalIntegerField};
use dirtybase_db_macro::DirtyTable;

use crate::cache_manager::cache_entry::CacheEntry;

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(table = "cache")]
pub struct CacheDbStoreEntity {
    id: OptionalIntegerField,
    #[dirty(flatten)]
    inner: CacheEntry,
}

impl From<CacheEntry> for CacheDbStoreEntity {
    fn from(value: CacheEntry) -> Self {
        Self {
            id: None,
            inner: value,
        }
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
