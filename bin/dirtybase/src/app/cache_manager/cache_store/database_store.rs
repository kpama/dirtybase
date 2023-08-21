use std::{collections::HashMap, sync::Arc};

use crate::app::{
    cache_manager::cache_entry::CacheEntry, entity::cache_db_store::CacheDbStoreRepository,
};

use super::CacheStoreTrait;
use async_trait::async_trait;

#[derive(Clone)]
pub struct DatabaseStore {
    repo: Arc<CacheDbStoreRepository>,
}

impl DatabaseStore {
    fn new(repo: CacheDbStoreRepository) -> Self {
        Self {
            repo: Arc::new(repo),
        }
    }
}

#[async_trait]
impl CacheStoreTrait for DatabaseStore {
    fn store_name() -> &'static str
    where
        Self: Sized,
    {
        "db"
    }

    /// Add the entry if it does not already exist
    async fn put(
        &self,
        key: &str,
        value: String,
        expiration: Option<i64>,
        tags: Option<&[&str]>,
    ) -> bool {
        self.repo.insert(key, &value, expiration).await
    }

    async fn put_many(
        &self,
        kv: &HashMap<String, String>,
        expiration: Option<i64>,
        tags: Option<&[&str]>,
    ) -> bool {
        self.repo.update_many(kv, expiration).await
    }

    // Add or replace existing entry
    async fn add(
        &self,
        key: &str,
        value: String,
        expiration: Option<i64>,
        tags: Option<&[&str]>,
    ) -> bool {
        self.repo.update(key, &value, expiration).await
    }

    async fn get(&self, key: &str) -> Option<CacheEntry> {
        let result = self.repo.get(key, false).await;
        result.map_or(None, |e| Some(CacheEntry::from(e)))
    }

    async fn many(&self, keys: &[&str]) -> Option<Vec<CacheEntry>> {
        self.repo.get_many(keys, false).await.map_or(None, |list| {
            Some(list.into_iter().map(|e| CacheEntry::from(e)).collect())
        })
    }

    // Delete an entry
    async fn forget(&self, key: &str) -> bool {
        self.repo.delete(key).await
    }

    // Delete all entries
    async fn flush(&self, tags: Option<&[&str]>) -> bool {
        self.repo.delete_all().await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for DatabaseStore {
    async fn inject(container: &busybody::ServiceContainer) -> Self {
        if let Some(manager) = container.get_type::<Self>() {
            return manager;
        } else {
            let repo: CacheDbStoreRepository = container.provide().await;
            let manager = Self::new(repo);
            return container.set_type(manager).get_type::<Self>().unwrap();
        }
    }
}
