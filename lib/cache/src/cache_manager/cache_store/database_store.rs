use std::{collections::HashMap, sync::Arc};

use crate::{
    cache_manager::cache_entry::CacheEntry, model::cache_db_store::CacheDbStoreRepository,
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
        key: String,
        value: String,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        self.repo.create(&key, &value, expiration, tags).await
    }

    async fn put_many(
        &self,
        kv: &HashMap<String, String>,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        self.repo.update_many(kv, expiration, tags).await
    }

    // Add or replace existing entry
    async fn add(
        &self,
        key: String,
        value: String,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        self.repo.update(&key, &value, expiration, tags).await
    }

    async fn get(&self, key: String) -> Option<CacheEntry> {
        let result = self.repo.get(&key, false).await;
        result.map(CacheEntry::from)
    }

    async fn many(&self, keys: &[String]) -> Option<Vec<CacheEntry>> {
        self.repo.get_many(keys, false).await.map(|list| list.into_iter().map(CacheEntry::from).collect())
    }

    // Delete an entry
    async fn forget(&self, key: String) -> bool {
        self.repo.delete(&key).await
    }

    // Delete all entries
    async fn flush(&self, tags: Option<&[String]>) -> bool {
        self.repo.delete_all(tags).await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for DatabaseStore {
    async fn inject(container: &busybody::ServiceContainer) -> Self {
        if let Some(store) = container.get_type::<Self>() {
            return store;
        } else {
            let repo: CacheDbStoreRepository = container.provide().await;
            let store = Self::new(repo);
            return container.set_type(store).get_type::<Self>().unwrap();
        }
    }
}
