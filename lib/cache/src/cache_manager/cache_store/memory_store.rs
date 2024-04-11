use super::CacheEntry;
use super::CacheStoreTrait;
use async_trait::async_trait;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
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

#[derive(Clone)]
pub struct MemoryStore {
    storage: Arc<RwLock<HashMap<String, Entry>>>,
    tags: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::default())),
            tags: Arc::new(RwLock::default()),
        }
    }

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
                        self.forget(an_entry.clone()).await;
                    }
                }
            }
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
        key: String,
        content: String,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        let mut lock = self.storage.write().await;
        lock.insert(
            key.clone(),
            Entry {
                expiration,
                content,
                key: key.clone(),
            },
        );

        self.tag_key(tags, key).await;

        return true;
    }

    async fn put_many(
        &self,
        kv: &HashMap<String, String>,
        duration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        for entry in kv {
            self.put(entry.0.clone(), entry.1.clone(), duration, tags)
                .await;
        }
        true
    }

    async fn get(&self, key: String) -> Option<CacheEntry> {
        let lock = self.storage.read().await;
        if let Some(entry) = lock.get(&key) {
            return Some(entry.into());
        }

        None
    }

    async fn many(&self, keys: &[String]) -> Option<Vec<CacheEntry>> {
        let mut results = Vec::<CacheEntry>::new();
        for a_key in keys {
            if let Some(entry) = self.get(a_key.into()).await {
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
        content: String,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        let mut lock = self.storage.write().await;
        self.tag_key(tags, key.clone()).await;

        if !lock.contains_key(&key) {
            lock.insert(
                key.clone(),
                Entry {
                    expiration,
                    content,
                    key,
                },
            );
            return true;
        }

        return false;
    }

    // Delete an entry
    async fn forget(&self, key: String) -> bool {
        let mut lock = self.storage.write().await;
        lock.remove(&key);
        true
    }

    // Delete all entries
    async fn flush(&self, tags: Option<&[String]>) -> bool {
        // Only delete on "tags" scope
        self.delete_tags(tags).await;

        true
    }
}

#[busybody::async_trait]
impl busybody::Injectable for MemoryStore {
    async fn inject(container: &busybody::ServiceContainer) -> Self {
        if let Some(store) = container.get_type::<Self>() {
            return store;
        } else {
            let store = Self::new();
            return container.set_type(store).get_type::<Self>().unwrap();
        }
    }
}
