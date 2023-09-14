use std::collections::HashMap;

use super::{CacheManager, FallbackFn};

pub struct CacheTagManager {
    tags: Vec<String>,
    cache_manager: CacheManager,
}

impl CacheTagManager {
    pub async fn new(tags: &[&str], cache_manager: CacheManager) -> Self {
        Self {
            tags: tags.iter().map(|entry| entry.to_string()).collect(),
            cache_manager,
        }
    }

    pub async fn store(self, name: &str) -> Self {
        Self {
            cache_manager: self.cache_manager.store(name).await,
            tags: self.tags,
        }
    }

    pub async fn tags(&self, tags: &[&str]) -> Self {
        self.cache_manager.tags(tags).await
    }

    pub fn cache_manager(&self) -> &CacheManager {
        &self.cache_manager
    }

    pub async fn add<V>(&self, key: &str, value: &V, expiration: Option<i64>) -> bool
    where
        V: serde::Serialize,
    {
        self.cache_manager
            .tag_and_add(&self.tags_to_ref(), key, value, expiration)
            .await
    }

    pub async fn put<V: serde::Serialize>(
        &self,
        key: &str,
        value: &V,
        expiration: Option<i64>,
    ) -> bool {
        self.cache_manager
            .tag_and_put(&self.tags_to_ref(), key, value, expiration)
            .await
    }

    pub async fn put_many<V: serde::Serialize>(
        &self,
        kv: &HashMap<String, V>,
        expiration: Option<i64>,
    ) -> bool {
        self.cache_manager
            .tag_and_put_many(&self.tags_to_ref(), kv, expiration)
            .await
    }

    pub async fn get<R: serde::de::DeserializeOwned>(&self, key: &str) -> Option<R> {
        self.cache_manager.get(key).await
    }

    pub async fn many<R: serde::de::DeserializeOwned>(
        &self,
        keys: &[&str],
    ) -> Option<HashMap<String, R>> {
        self.cache_manager.many(keys).await
    }

    pub async fn has(&self, key: &str) -> bool {
        self.cache_manager.has(key).await
    }

    pub async fn flush(&self) -> bool {
        self.cache_manager.flush_tags(&self.tags_to_ref()).await
    }

    pub async fn increment(&self, key: &str) -> bool {
        self.cache_manager.increment(key).await
    }

    pub async fn increment_by(&self, key: &str, by: f64) -> bool {
        self.cache_manager.increment_by(key, by).await
    }

    pub async fn decrement(&self, key: &str) -> bool {
        self.cache_manager.decrement(key).await
    }

    pub async fn decrement_by(&self, key: &str, by: f64) -> bool {
        self.cache_manager.decrement_by(key, by).await
    }

    pub async fn remember<F, R>(&self, key: &str, expiration: Option<i64>, default: F) -> R
    where
        R: serde::Serialize + serde::de::DeserializeOwned,
        F: FallbackFn<(), R>,
    {
        self.cache_manager
            .do_remember(key, expiration, default, Some(&self.tags_to_ref()))
            .await
    }

    pub async fn remember_forever<F, R>(&self, key: &str, default: F) -> R
    where
        R: serde::Serialize + serde::de::DeserializeOwned,
        F: FallbackFn<(), R>,
    {
        self.remember(key, None, default).await
    }

    fn tags_to_ref(&self) -> Vec<&str> {
        self.tags.iter().map(|s| s.as_ref()).collect::<Vec<&str>>()
    }
}
