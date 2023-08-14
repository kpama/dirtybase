use self::cache_store::{CacheStoreTrait, DatabaseStore, MemoryStore};
use busybody::helpers::provide;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

pub mod cache_entry;
pub mod cache_store;

type StoreDriver = Arc<Box<dyn CacheStoreTrait + Send + Sync>>;

#[derive(Clone)]
pub struct CacheManager {
    stores: HashMap<String, StoreDriver>,
    default_store: String,
}

impl CacheManager {
    pub async fn new() -> Self {
        // TODO: Move some of this logic outside of the method
        //       The application suppose to pass the list of stores'
        //       Drivers
        let mut stores: HashMap<String, StoreDriver> = HashMap::new();
        let default_store = MemoryStore::store_name().to_string();

        stores.insert(
            default_store.clone(),
            Arc::new(Box::new(MemoryStore::new())),
        );
        stores.insert(
            DatabaseStore::store_name().into(),
            Arc::new(Box::new(provide::<DatabaseStore>().await)),
        );

        Self {
            stores,
            default_store,
        }
    }

    fn make(store: StoreDriver, name: &str) -> Self {
        let mut stores = HashMap::new();
        stores.insert(name.to_string(), store);

        Self {
            stores,
            default_store: name.into(),
        }
    }

    pub fn store(&self, name: &str) -> Self {
        Self::make(
            if let Some(store) = self.stores.get(name) {
                store.clone()
            } else {
                self.default_store()
            },
            name,
        )
    }

    pub fn default_store(&self) -> StoreDriver {
        self.stores.get(&self.default_store).unwrap().clone()
    }

    pub fn has_store(&self, name: &str) -> bool {
        self.stores.contains_key(name)
    }

    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        if let Some(value) = self.default_store().get(key).await {
            return match serde_json::from_str(&value.value) {
                Ok(v) => Some(v),
                _ => None,
            };
        }

        None
    }

    pub async fn many(&self, keys: &[&str]) -> Option<HashMap<String, serde_json::Value>> {
        if let Some(map) = self.default_store().many(keys).await {
            let mut built = HashMap::new();
            for entry in map {
                if let Ok(value) = serde_json::from_str(&entry.value) {
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
    pub async fn add<V>(&self, key: &str, value: &V, duration: Option<i64>) -> bool
    where
        V: serde::Serialize,
    {
        dbg!("ts: {:#?}", &duration);

        match serde_json::to_string(value) {
            Ok(v) => self.default_store().add(key, v, duration).await,
            _ => false,
        }
    }

    /// Check is an entry exist
    pub async fn has(&self, key: &str) -> bool {
        self.default_store().get(key).await.is_some()
    }

    // Fetches and deletes the entry with the key specified
    pub async fn pull(&self, key: &str) -> Option<serde_json::Value> {
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
    pub async fn remember<F, R>(
        &self,
        key: &str,
        expiration: Option<i64>,
        default: F,
    ) -> serde_json::Value
    where
        R: serde::Serialize,
        F: FallbackFn<(), R>,
    {
        let value = self.get(key).await;
        if value.is_none() {
            let result = serde_json::to_value(default.call(()).await).unwrap();
            self.put(key, &result, expiration).await;
            return result;
        } else {
            value.unwrap()
        }
    }

    pub async fn remember_forever<F, R>(&self, key: &str, default: F) -> serde_json::Value
    where
        R: serde::Serialize,
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
        match serde_json::to_string(value) {
            Ok(v) => self.default_store().put(key, v, expiration).await,
            _ => false,
        }
    }

    pub async fn put_many<V: serde::Serialize>(
        &self,
        kv: &HashMap<String, V>,
        expiration: Option<i64>,
    ) -> bool {
        let built = kv
            .iter()
            .map(|entry| (entry.0.clone(), serde_json::to_string(entry.1)))
            .filter(|entry| entry.1.is_ok())
            .map(|entry| (entry.0, entry.1.unwrap()))
            .collect();

        self.default_store().put_many(&built, expiration).await
    }

    pub async fn forever<V: serde::Serialize>(&self, key: &str, value: &V) -> bool {
        self.put(key, value, None).await
    }

    /// Deletes the entry for this key
    pub async fn forget(&self, key: &str) -> bool {
        self.default_store().forget(key).await
    }

    /// Delete everything in the store
    pub async fn flush(&self) -> bool {
        self.default_store().flush().await
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
                        return self.default_store().put(key, real.to_string(), None).await;
                    }
                    false
                }
                _ => false,
            };
        }
        false
    }
}

#[busybody::async_trait]
impl busybody::Injectable for CacheManager {
    async fn inject(container: &busybody::ServiceContainer) -> Self {
        if let Some(manager) = container.get_type::<CacheManager>() {
            return manager;
        } else {
            let manager = Self::new().await;
            return container
                .set_type(manager)
                .get_type::<CacheManager>()
                .unwrap();
        }
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
