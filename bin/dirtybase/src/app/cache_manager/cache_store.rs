use std::collections::HashMap;

use async_trait::async_trait;

mod database_store;
mod memory_store;
mod redis_store;

pub use database_store::DatabaseStore;
pub use memory_store::MemoryStore;
pub use redis_store::RedisStore;

use super::cache_entry::CacheEntry;

#[async_trait]
pub trait CacheStoreTrait: Send + Sync {
    fn store_name() -> &'static str
    where
        Self: Sized;

    async fn get(&self, key: String) -> Option<CacheEntry>;

    async fn many(&self, keys: &[String]) -> Option<Vec<CacheEntry>>;

    /// Add the entry if it does not already exist
    async fn put(
        &self,
        key: String,
        value: String,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool;

    async fn put_many(
        &self,
        kv: &HashMap<String, String>,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool;

    // Add or replace existing entry
    async fn add(
        &self,
        key: String,
        value: String,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool;

    // Delete an entry
    async fn forget(&self, key: String) -> bool;

    // Delete all entries
    async fn flush(&self, tags: Option<&[String]>) -> bool;
}
