use super::CacheEntry;
use super::CacheStoreTrait;
use async_trait::async_trait;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct MemoryStore {
    storage: Arc<RwLock<HashMap<String, CacheEntry>>>,
    tags: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

impl MemoryStore {
    async fn tag_key(&self, tags: Option<&[String]>, key: String) {
        let mut lock = self.tags.write().await;
        if let Some(list) = tags {
            for a_tag in list {
                if !lock.contains_key(a_tag) {
                    lock.insert(a_tag.clone(), HashSet::default());
                }

                if let Some(set) = lock.get_mut(a_tag) {
                    set.insert(key.clone());
                }
            }
        }
    }

    async fn delete_tags(&self, tags: Option<&[String]>) {
        let mut lock = self.tags.write().await;

        if let Some(list) = tags {
            for a_tag in list {
                if let Some(set) = lock.get_mut(a_tag) {
                    for an_entry in set.iter() {
                        self.forget(&an_entry).await;
                    }
                }
            }
        }
    }
}

#[async_trait]
impl CacheStoreTrait for MemoryStore {
    /// Add the entry if it does not already exist
    async fn put(
        &self,
        key: String,
        content: serde_json::Value,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        let data = CacheEntry::new(key.clone(), content, expiration);
        let mut lock = self.storage.write().await;
        lock.insert(key.clone(), data);

        self.tag_key(tags, key.to_string()).await;

        true
    }

    async fn put_many(
        &self,
        kv: HashMap<String, serde_json::Value>,
        duration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        for entry in kv {
            if !self.put(entry.0, entry.1, duration, tags).await {
                return false;
            }
        }
        true
    }

    async fn get(&self, key: &str) -> Option<CacheEntry> {
        let r_lock = self.storage.read().await;
        r_lock.get(key).cloned()
    }

    async fn many(&self, keys: &[String]) -> Option<Vec<CacheEntry>> {
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
        key: String,
        content: serde_json::Value,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        let mut lock = self.storage.write().await;
        self.tag_key(tags, key.to_string()).await;

        if !lock.contains_key(&key) {
            lock.insert(key.to_string(), CacheEntry::new(key, content, expiration));
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
    async fn flush(&self, tags: Option<&[String]>) -> bool {
        // Only delete on "tags" scope
        self.delete_tags(tags).await;

        true
    }
}
