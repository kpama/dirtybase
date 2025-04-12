use std::{collections::HashMap, sync::Arc};

use crate::{CacheEntry, cache_manager::cache_store::CacheStoreTrait};

pub struct CacheStorageProvider {
    inner: Arc<Box<dyn CacheStoreTrait>>,
}

#[async_trait::async_trait]
impl CacheStoreTrait for CacheStorageProvider {
    async fn get(&self, key: &str) -> Option<CacheEntry> {
        self.inner.get(key).await
    }

    async fn many(&self, keys: &[String]) -> Option<Vec<CacheEntry>> {
        self.inner.many(keys).await
    }

    async fn put(
        &self,
        key: String,
        value: serde_json::Value,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        self.inner.put(key, value, expiration, tags).await
    }

    async fn put_many(
        &self,
        kv: HashMap<String, serde_json::Value>,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        self.inner.put_many(kv, expiration, tags).await
    }

    async fn add(
        &self,
        key: String,
        value: serde_json::Value,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        self.inner.add(key, value, expiration, tags).await
    }

    async fn forget(&self, key: &str) -> bool {
        self.inner.forget(key).await
    }

    async fn flush(&self, tags: Option<&[String]>) -> bool {
        self.inner.flush(tags).await
    }
}
