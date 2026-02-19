use std::collections::HashMap;

use crate::{
    CacheStorageResolver, cache_manager::cache_entry::CacheEntry,
    model::cache_db_store::CacheDbStoreRepository,
};

use super::CacheStoreTrait;
use async_trait::async_trait;
use dirtybase_common::db::base::manager::Manager;

#[derive(Clone)]
pub struct DatabaseStore {
    repo: CacheDbStoreRepository,
}

impl DatabaseStore {
    pub async fn register() {
        CacheStorageResolver::register("database", |mut resolver| async move {
            let manager = resolver
                .context_ref()
                .get::<Manager>()
                .await
                .expect("could not get db manager");
            resolver.set_storage(Self {
                repo: CacheDbStoreRepository::new(manager),
            });

            resolver
        })
        .await;
    }
}

#[async_trait]
impl CacheStoreTrait for DatabaseStore {
    /// Add the entry if it does not already exist
    async fn put(
        &self,
        key: String,
        value: serde_json::Value,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        self.repo.create(key, value, expiration, tags).await
    }

    async fn put_many(
        &self,
        kv: HashMap<String, serde_json::Value>,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        self.repo.update_many(kv, expiration, tags).await
    }

    // Add or replace existing entry
    async fn add(
        &self,
        key: String,
        value: serde_json::Value,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        self.repo.update(key, value, expiration, tags).await
    }

    async fn get(&self, key: &str) -> Option<CacheEntry> {
        let result = self.repo.get(key, false).await;
        result.map(CacheEntry::from)
    }

    async fn many(&self, keys: &[String]) -> Option<Vec<CacheEntry>> {
        self.repo
            .get_many(keys, false)
            .await
            .map(|list| list.into_iter().map(CacheEntry::from).collect())
    }

    // Delete an entry
    async fn forget(&self, key: &str) -> bool {
        self.repo.delete(key).await
    }

    // Delete all entries
    async fn flush(&self, tags: Option<&[String]>) -> bool {
        self.repo.delete_all(tags).await
    }
}
