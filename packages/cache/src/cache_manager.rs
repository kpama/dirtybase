use crate::CacheStorageProvider;

use self::cache_store::CacheStoreTrait;
use std::collections::HashMap;
use std::future::Future;

use dirtybase_helper::time::{Time, now};

pub mod cache_entry;
pub mod cache_store;

#[derive(Clone)]
pub struct CacheManager {
    store: CacheStorageProvider,
    prefix: Option<String>,
    tags: Vec<String>,
}

impl CacheManager {
    pub fn new(storage: CacheStorageProvider, prefix: Option<String>) -> Self {
        Self {
            prefix,
            store: storage,
            tags: Vec::new(),
        }
    }

    pub async fn prefix(&self, prefix: &str) -> Self {
        Self {
            store: self.store.clone(),
            prefix: Some(prefix.to_string()),
            tags: Vec::new(),
        }
    }

    pub async fn get<R>(&self, key: &str) -> Option<R>
    where
        R: serde::de::DeserializeOwned,
    {
        let key = self.prefix_a_key(key);
        if let Some(entry) = self.store.get(&key).await
            && entry.still_hot()
        {
            return match serde_json::from_value(entry.value) {
                Ok(v) => Some(v),
                Err(e) => {
                    log::error!("Error parsing cache data. {e}");
                    None
                }
            };
        }

        None
    }

    fn prefix_keys(&self, keys: &[&str]) -> Vec<String> {
        let prefix = self.prefix.as_ref().unwrap_or(&"core".to_string()).clone();

        keys.iter()
            .map(|e| prefix.clone() + ":" + *e)
            .collect::<Vec<String>>()
    }

    fn prefix_tags(&self, tags: Option<&[&str]>) -> Option<Vec<String>> {
        tags.map(|t| self.prefix_keys(t))
    }

    fn prefix_a_key(&self, key: &str) -> String {
        let prefix = self.prefix.as_ref().unwrap_or(&"core".to_string()).clone();
        dirtybase_helper::hash::sha256::hash_string(prefix + ":" + key)
    }

    pub async fn tags(&self, tags: &[&str]) -> Self {
        Self {
            store: self.store.clone(),
            tags: tags.iter().map(|entry| entry.to_string()).collect(),
            prefix: self.prefix.clone(),
        }
    }

    pub async fn many<R: serde::de::DeserializeOwned>(
        &self,
        keys: &[&str],
    ) -> Option<HashMap<String, R>> {
        let the_keys = self.prefix_keys(keys);

        if let Some(map) = self.store.many(&the_keys).await {
            let mut built = HashMap::new();
            for entry in map {
                if entry.still_hot()
                    && let Ok(value) = serde_json::from_value(entry.value)
                {
                    built.insert(entry.key.clone(), value);
                }
            }

            if !built.is_empty() {
                return Some(built);
            }
        }
        None
    }

    // Tries to add the value if the value does not already exist
    pub async fn add(
        &self,
        key: &str,
        value: impl serde::Serialize,
        expiration: Option<i64>,
    ) -> bool {
        self.do_add(key, value, expiration, Some(&self.tags_to_ref()))
            .await
    }

    /// Check is an entry exist
    pub async fn has(&self, key: &str) -> bool {
        let key = self.prefix_a_key(key);
        self.store.get(&key).await.is_some()
    }

    // Fetches and deletes the entry with the key specified
    pub async fn pull<R: serde::de::DeserializeOwned>(&self, key: &str) -> Option<R> {
        let value = self.get(key).await;
        if value.is_some() {
            self.forget(key).await;
        }

        value
    }

    pub async fn increment(&self, key: &str) -> bool {
        self.increment_by(key, 1.0).await
    }

    pub async fn increment_by(&self, key: &str, by: f64) -> bool {
        self.inc_or_dec(key, by, true).await
    }

    pub async fn decrement(&self, key: &str) -> bool {
        self.decrement_by(key, 1.0).await
    }

    pub async fn decrement_by(&self, key: &str, by: f64) -> bool {
        self.inc_or_dec(key, by, false).await
    }

    /// Try fetching the stored value or fallback to the default
    pub async fn remember<F, R>(&self, key: &str, expiration: Option<i64>, default: F) -> R
    where
        R: serde::Serialize + serde::de::DeserializeOwned,
        F: FallbackFn<(), R>,
    {
        self.do_remember(key, expiration, default, Some(&self.tags_to_ref()))
            .await
    }

    pub async fn remember_forever<F, R>(&self, key: &str, default: F) -> R
    where
        R: serde::Serialize + serde::de::DeserializeOwned,
        F: FallbackFn<(), R>,
    {
        self.remember(key, None, default).await
    }

    /// Store the passed value
    pub async fn put<V: serde::Serialize>(
        &self,
        key: &str,
        value: &V,
        expiration: Option<i64>,
    ) -> bool {
        self.do_put(key, value, expiration, Some(&self.tags_to_ref()))
            .await
    }

    pub async fn put_many<V: serde::Serialize>(
        &self,
        kv: &HashMap<String, V>,
        expiration: Option<i64>,
    ) -> bool {
        self.do_put_many(kv, expiration, Some(&self.tags_to_ref()))
            .await
    }

    pub async fn forever<V: serde::Serialize>(&self, key: &str, value: &V) -> bool {
        self.put(key, value, None).await
    }

    /// Deletes the entry for this key
    pub async fn forget(&self, key: &str) -> bool {
        self.store.forget(key).await
    }

    /// Delete everything in the store
    pub async fn flush(&self) -> bool {
        self.store.flush(None).await
    }

    pub async fn flush_tags(&self, tags: &[&str]) -> bool {
        let tags = self.prefix_tags(Some(tags));
        self.store.flush(tags.as_deref()).await
    }

    async fn inc_or_dec(&self, key: &str, by: f64, incrementing: bool) -> bool {
        let value = self.get(key).await;
        if value.is_some() {
            return match value {
                Some(serde_json::Value::Number(n)) => {
                    if let Some(mut real) = n.as_f64() {
                        if incrementing {
                            real += by;
                        } else {
                            real -= by;
                        }

                        return self.do_add(key, &real.to_string(), None, None).await;
                    }
                    false
                }
                _ => false,
            };
        }
        false
    }

    pub fn now(&self) -> Time {
        now()
    }

    pub(super) async fn do_add(
        &self,
        key: &str,
        value: impl serde::Serialize,
        expiration: Option<i64>,
        tags: Option<&[&str]>,
    ) -> bool {
        let key = self.prefix_a_key(key);
        let tags = self.prefix_tags(tags);

        match serde_json::to_value(&value) {
            Ok(v) => self.store.add(key, v, expiration, tags.as_deref()).await,
            _ => false,
        }
    }

    pub(super) async fn do_put<V: serde::Serialize>(
        &self,
        key: &str,
        value: &V,
        expiration: Option<i64>,
        tags: Option<&[&str]>,
    ) -> bool {
        let key = self.prefix_a_key(key);
        let tags = self.prefix_tags(tags);

        match serde_json::to_value(value) {
            Ok(v) => self.store.put(key, v, expiration, tags.as_deref()).await,
            Err(e) => {
                log::error!("{e}");
                false
            }
        }
    }

    pub(super) async fn do_put_many<V: serde::Serialize>(
        &self,
        kv: &HashMap<String, V>,
        expiration: Option<i64>,
        tags: Option<&[&str]>,
    ) -> bool {
        let tags = self.prefix_tags(tags);

        let built = kv
            .iter()
            .map(|entry| (entry.0.clone(), serde_json::to_value(entry.1)))
            .filter(|entry| entry.1.is_ok())
            .map(|entry| (entry.0, entry.1.unwrap()))
            .collect();

        self.store
            .put_many(built, expiration, tags.as_deref())
            .await
    }

    pub async fn do_remember<F, R>(
        &self,
        key: &str,
        expiration: Option<i64>,
        default: F,
        tags: Option<&[&str]>,
    ) -> R
    where
        R: serde::Serialize + serde::de::DeserializeOwned,
        F: FallbackFn<(), R>,
    {
        if let Some(value) = self.get(key).await {
            value
        } else {
            let new_value = default.call(()).await;
            let result = serde_json::to_value(&new_value).unwrap();
            self.do_put(key, &result, expiration, tags).await;
            new_value
        }
    }

    fn tags_to_ref(&self) -> Vec<&str> {
        self.tags.iter().map(|s| s.as_ref()).collect::<Vec<&str>>()
    }
}

pub trait FallbackFn<Args, R: serde::Serialize>: Send + Sync + 'static {
    type Future: Future<Output = R> + Send;

    fn call(&self, args: Args) -> Self::Future;
}

impl<Func, Fut, R> FallbackFn<(), R> for Func
where
    R: serde::Serialize,
    Func: Send + Sync + Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send,
{
    type Future = Fut;
    fn call(&self, _: ()) -> Self::Future {
        (self)()
    }
}
