use dirtybase_common::db::types::{CreatedAtField, OptionalStringField, UpdatedAtField};
use dirtybase_contract::db_contract::types::OptionalIntegerField;
use dirtybase_db_macro::DirtyTable;

use crate::cache_manager::cache_entry::CacheEntry;

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(table = "cache", no_soft_delete)]
pub struct CacheDbStoreEntity {
    id: OptionalIntegerField,
    #[dirty(flatten)]
    inner: CacheEntry,
    #[dirty(skip_select)]
    tag: OptionalStringField,
    created_at: CreatedAtField,
    updated_at: UpdatedAtField,
}

impl CacheDbStoreEntityRepo {
    pub async fn by_key(&mut self, key: &str) -> Result<Option<CacheDbStoreEntity>, anyhow::Error> {
        self.filter(|qb| {
            // qb.is_eq(Self::, value)
        });

        Ok(None)
    }
}

impl From<CacheEntry> for CacheDbStoreEntity {
    fn from(value: CacheEntry) -> Self {
        Self {
            id: None,
            inner: value,
            tag: None,
            created_at: None,
            updated_at: None,
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
