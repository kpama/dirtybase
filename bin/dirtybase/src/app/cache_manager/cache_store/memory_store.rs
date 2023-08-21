use crate::app::cache_manager::cache_entry::CacheEntry;

use super::CacheStoreTrait;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
struct Entry {
    expiration: Option<i64>,
    content: String,
    key: String,
}

impl From<Entry> for CacheEntry {
    fn from(value: Entry) -> Self {
        Self::new(&value.key, &value.content, value.expiration)
    }
}

impl From<&Entry> for CacheEntry {
    fn from(value: &Entry) -> Self {
        Self::new(&value.key, &value.content, value.expiration)
    }
}

impl From<CacheEntry> for Entry {
    fn from(value: CacheEntry) -> Self {
        Self {
            key: value.key,
            expiration: value.expiration,
            content: value.value,
        }
    }
}

pub struct MemoryStore {
    storage: RwLock<HashMap<String, Entry>>,
    tags: RwLock<HashMap<String, Vec<String>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            storage: RwLock::new(HashMap::default()),
            tags: RwLock::default(),
        }
    }
}

#[async_trait]
impl CacheStoreTrait for MemoryStore {
    fn store_name() -> &'static str
    where
        Self: Sized,
    {
        "memory"
    }

    /// Add the entry if it does not already exist
    async fn put(
        &self,
        key: &str,
        content: String,
        expiration: Option<i64>,
        tags: Option<&[&str]>,
    ) -> bool {
        let mut lock = self.storage.write().await;
        lock.insert(
            key.into(),
            Entry {
                expiration,
                content,
                key: key.into(),
            },
        );
        return true;
    }

    async fn put_many(
        &self,
        kv: &HashMap<String, String>,
        duration: Option<i64>,
        tags: Option<&[&str]>,
    ) -> bool {
        for entry in kv {
            self.put(entry.0, entry.1.clone(), duration, tags).await;
        }
        true
    }

    async fn get(&self, key: &str) -> Option<CacheEntry> {
        let lock = self.storage.read().await;
        if let Some(entry) = lock.get(key) {
            return Some(entry.into());
        }

        None
    }

    async fn many(&self, keys: &[&str]) -> Option<Vec<CacheEntry>> {
        let mut results = Vec::<CacheEntry>::new();
        for a_key in keys {
            if let Some(entry) = self.get(&a_key).await {
                results.push(entry);
            }
        }

        if results.is_empty() {
            None
        } else {
            Some(results)
        }
    }

    // Add or replace existing entry
    async fn add(
        &self,
        key: &str,
        content: String,
        expiration: Option<i64>,
        tags: Option<&[&str]>,
    ) -> bool {
        let mut lock = self.storage.write().await;
        if !lock.contains_key(key) {
            lock.insert(
                key.into(),
                Entry {
                    expiration,
                    content,
                    key: key.to_string(),
                },
            );
            return true;
        }

        return false;
    }

    // Delete an entry
    async fn forget(&self, key: &str) -> bool {
        let mut lock = self.storage.write().await;
        lock.remove(key);
        true
    }

    // Delete all entries
    async fn flush(&self, tags: Option<&[&str]>) -> bool {
        let mut lock = self.storage.write().await;
        lock.drain();
        true
    }
}
