use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use redis::AsyncCommands;
use tokio::sync::RwLock;

use crate::cache_manager::cache_entry::CacheEntry;

use super::CacheStoreTrait;

#[derive(Clone)]
pub struct RedisStore {
    redis_client: Arc<RwLock<redis::aio::MultiplexedConnection>>,
}

impl RedisStore {
    async fn tag_key(&self, key: &str, tags: Option<&[String]>) {
        if tags.is_some() {
            let mut client = self.redis_client.write().await;
            for a_tag in tags.unwrap() {
                _ = client.sadd::<&String, &str, bool>(a_tag, key).await;
            }
        }
    }
}

#[async_trait]
impl CacheStoreTrait for RedisStore {
    async fn get(&self, key: &str) -> Option<CacheEntry> {
        let mut client = self.redis_client.write().await;

        if let Ok(data) = client.get::<String, String>(key.to_string()).await {
            return serde_json::from_str::<CacheEntry>(&data).ok();
        }

        None
    }

    async fn many(&self, keys: &[String]) -> Option<Vec<CacheEntry>> {
        let mut client = self.redis_client.write().await;

        let ks = keys.iter().map(|e| e.to_string()).collect::<Vec<String>>();
        if let Ok(list_of_data) = client.mget::<&[String], Vec<String>>(&ks).await {
            return Some(
                list_of_data
                    .iter()
                    .flat_map(|entry| serde_json::from_str::<CacheEntry>(entry))
                    .collect::<Vec<CacheEntry>>(),
            );
        }

        None
    }

    async fn put(
        &self,
        key: String,
        value: serde_json::Value,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        let mut client = self.redis_client.write().await;
        let entry = CacheEntry::new(key.clone(), value, expiration);

        if let Ok(data) = serde_json::to_string(&entry) {
            let result = if expiration.is_some() {
                let options = redis::SetOptions::default()
                    .with_expiration(redis::SetExpiry::EX(expiration.unwrap() as u64));
                client
                    .set_options::<&String, String, bool>(&key, data, options)
                    .await
                    .is_ok()
            } else {
                client
                    .set::<&String, String, bool>(&key.to_string(), data)
                    .await
                    .is_ok()
            };

            self.tag_key(&key, tags).await;

            return result;
        }

        false
    }

    async fn put_many(
        &self,
        kv: HashMap<String, serde_json::Value>,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        for entry in kv {
            if !self.put(entry.0, entry.1.clone(), expiration, tags).await {
                return false;
            }
        }

        true
    }

    async fn add(
        &self,
        key: String,
        value: serde_json::Value,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        let mut client = self.redis_client.write().await;
        let entry = CacheEntry::new(key.clone(), value, expiration);

        if let Ok(data) = serde_json::to_string(&entry) {
            let options = if expiration.is_some() {
                redis::SetOptions::default()
                    .conditional_set(redis::ExistenceCheck::NX)
                    .with_expiration(redis::SetExpiry::EX(expiration.unwrap() as u64))
            } else {
                redis::SetOptions::default().conditional_set(redis::ExistenceCheck::NX)
            };

            let result = client
                .set_options::<&String, String, bool>(&key, data, options)
                .await
                .is_ok();

            self.tag_key(&key, tags).await;

            return result;
        }

        false
    }

    async fn forget(&self, key: &str) -> bool {
        let mut client = self.redis_client.write().await;

        _ = client.unlink::<String, bool>(key.to_string()).await;

        true
    }

    async fn flush(&self, tags: Option<&[String]>) -> bool {
        // At the moment, we are not going to flush/delete all entries in the database
        if tags.is_some() {
            let mut client = self.redis_client.write().await;
            for a_tag in tags.unwrap() {
                if let Ok(lists) = client.smembers::<&String, Vec<String>>(a_tag).await {
                    for a_key in lists {
                        _ = client.unlink::<String, bool>(a_key).await;
                    }
                }
            }
        }

        return true;
    }
}
