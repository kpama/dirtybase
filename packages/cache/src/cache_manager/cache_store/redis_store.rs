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
    pub fn new(conn: redis::aio::MultiplexedConnection) -> Self {
        Self {
            redis_client: Arc::new(RwLock::new(conn)),
        }
    }

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
    fn store_name() -> &'static str
    where
        Self: Sized,
    {
        "redis"
    }

    async fn get(&self, key: String) -> Option<CacheEntry> {
        let mut client = self.redis_client.write().await;

        if let Ok(data) = client.get::<String, String>(key).await {
            return serde_json::from_str::<CacheEntry>(&data).ok();
        }

        None
    }

    async fn many(&self, keys: &[String]) -> Option<Vec<CacheEntry>> {
        let mut client = self.redis_client.write().await;

        if let Ok(list_of_data) = client.mget::<&[String], Vec<String>>(keys).await {
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
        value: String,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        let mut client = self.redis_client.write().await;
        let entry = CacheEntry::new(&key, &value, expiration);

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
                    .set::<&String, String, bool>(&key, data)
                    .await
                    .is_ok()
            };

            self.tag_key(key.as_str(), tags).await;

            return result;
        }

        false
    }

    async fn put_many(
        &self,
        kv: &HashMap<String, String>,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        for entry in kv {
            _ = self
                .put(entry.0.clone(), entry.1.clone(), expiration, tags)
                .await;
        }

        false
    }

    async fn add(
        &self,
        key: String,
        value: String,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        let mut client = self.redis_client.write().await;
        let entry = CacheEntry::new(&key, &value, expiration);

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

    async fn forget(&self, key: String) -> bool {
        let mut client = self.redis_client.write().await;

        _ = client.unlink::<String, bool>(key).await;

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

#[busybody::async_trait]
impl busybody::Injectable for RedisStore {
    async fn inject(container: &busybody::ServiceContainer) -> Self {
        if let Some(store) = container.get_type::<Self>().await {
            return store;
        } else {
            let redis_client = dirtybase_3rd_client::redis::get_client().await.unwrap();
            let store = Self::new(redis_client);
            return container
                .set_type(store)
                .await
                .get_type::<Self>()
                .await
                .unwrap();
        }
    }
}
