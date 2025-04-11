use std::sync::Arc;

use crate::{CacheEntry, cache_manager::cache_store::CacheStoreTrait};

pub struct CacheStorageProvider {
    inner: Arc<Box<dyn CacheStoreTrait>>,
}

// #[async_trait::async_trait]
// impl CacheStoreTrait for CacheStorageProvider {
//     async fn get(&self, key: &str) -> Option<CacheEntry> {
//         self.inner.get(key).await
//     }

//     async fn many(&self, keys: &[&str]) -> Option<Vec<CacheEntry>> {
//         self.inner.many(keys).await
//     }

//     async fn put<V>(
//         &self,
//         key: &str,
//         value: V,
//         expiration: Option<i64>,
//         tags: Option<&[String]>,
//     ) -> bool
//     where
//         V: serde::Serialize,
//     {
//     }
// }
