use std::collections::HashMap;

use busybody::helpers::provide;

use super::CacheManager;

pub struct CacheTagManager {
    tags: Vec<String>,
    cache_manager: CacheManager,
}

impl CacheTagManager {
    pub async fn new(tags: &[&str]) -> Self {
        let cache_manager = provide::<CacheManager>().await;
        Self {
            tags: tags.iter().map(|entry| entry.to_string()).collect(),
            cache_manager,
        }
    }

    pub fn store(self, name: &str) -> Self {
        Self {
            cache_manager: self.cache_manager.store(name),
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

    pub async fn flush(&self) -> bool {
        self.cache_manager.flush_tags(&self.tags_to_ref()).await
    }

    fn tags_to_ref(&self) -> Vec<&str> {
        self.tags.iter().map(|s| s.as_ref()).collect::<Vec<&str>>()
    }
}
