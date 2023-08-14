use std::collections::HashMap;

use async_trait::async_trait;

mod database_store;
mod memory_store;

pub use database_store::DatabaseStore;
pub use memory_store::MemoryStore;

use super::cache_entry::CacheEntry;

/*
  [x] public function get($key) {}
  [x] public function many(array $keys) {}
  [x] public function put($key, $value, $seconds) {}
  [x] public function putMany(array $values, $seconds) {}
  [x] (implemented in the cache store manager) public function increment($key, $value = 1) {}
  [x] (implemented in the cache store manager) public function decrement($key, $value = 1) {}
  [x] (implemented in the cache store manager) public function forever($key, $value) {}
  [x]  public function forget($key) {}
  [x] (flush implemented as `flush`) public function flush() {}
  [] (implemented in the cache store manager) public function getPrefix() {}
*/

#[async_trait]
pub trait CacheStoreTrait: Send + Sync {
    fn store_name() -> &'static str
    where
        Self: Sized;

    async fn get(&self, key: &str) -> Option<CacheEntry>;

    async fn many(&self, keys: &[&str]) -> Option<Vec<CacheEntry>>;

    /// Add the entry if it does not already exist
    async fn put(&self, key: &str, value: String, duration: Option<i64>) -> bool;

    async fn put_many(&self, kv: &HashMap<String, String>, duration: Option<i64>) -> bool;

    // Add or replace existing entry
    async fn add(&self, key: &str, value: String, duration: Option<i64>) -> bool;

    // Delete an entry
    async fn forget(&self, key: &str) -> bool;

    // Delete all entries
    async fn flush(&self) -> bool;
}
