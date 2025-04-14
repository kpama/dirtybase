use std::{collections::HashMap, sync::Arc};

use dirtybase_contract::prelude::Context;

use crate::{CacheEntry, cache_manager::cache_store::CacheStoreTrait, config::CacheConfig};

#[derive(Clone)]
pub struct CacheStorageProvider {
    inner: Arc<Box<dyn CacheStoreTrait>>,
}

#[async_trait::async_trait]
impl CacheStoreTrait for CacheStorageProvider {
    async fn get(&self, key: &str) -> Option<CacheEntry> {
        self.inner.get(key).await
    }

    async fn many(&self, keys: &[String]) -> Option<Vec<CacheEntry>> {
        self.inner.many(keys).await
    }

    async fn put(
        &self,
        key: String,
        value: serde_json::Value,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        self.inner.put(key, value, expiration, tags).await
    }

    async fn put_many(
        &self,
        kv: HashMap<String, serde_json::Value>,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        self.inner.put_many(kv, expiration, tags).await
    }

    async fn add(
        &self,
        key: String,
        value: serde_json::Value,
        expiration: Option<i64>,
        tags: Option<&[String]>,
    ) -> bool {
        self.inner.add(key, value, expiration, tags).await
    }

    async fn forget(&self, key: &str) -> bool {
        self.inner.forget(key).await
    }

    async fn flush(&self, tags: Option<&[String]>) -> bool {
        self.inner.flush(tags).await
    }
}

impl CacheStorageProvider {
    pub fn from<S>(storage: S) -> Self
    where
        S: CacheStoreTrait + 'static,
    {
        Self {
            inner: Arc::new(Box::new(storage)),
        }
    }
}

pub struct CacheStorageResolver {
    context: Context,
    provider: Option<CacheStorageProvider>,
    config: CacheConfig,
}

impl CacheStorageResolver {
    pub fn new(context: Context, config: CacheConfig) -> Self {
        Self {
            config,
            context,
            provider: None,
        }
    }

    pub fn has_provider(&self) -> bool {
        self.provider.is_some()
    }

    pub fn context_ref(&self) -> &Context {
        &self.context
    }

    pub fn config_ref(&self) -> &CacheConfig {
        &self.config
    }

    pub fn context(&self) -> Context {
        self.context.clone()
    }

    pub fn set_storage(&mut self, storage: impl CacheStoreTrait + 'static) {
        self.provider = Some(CacheStorageProvider::from(storage));
    }

    pub async fn get_provider(self) -> Result<CacheStorageProvider, anyhow::Error> {
        match Self::get_middleware().await.send(self).await.provider {
            Some(p) => Ok(p),
            None => Err(anyhow::anyhow!(
                "could not resolve the cache storage provider"
            )),
        }
    }

    pub async fn register<F, Fut>(name: &str, callback: F)
    where
        F: Clone + Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Self> + Send + 'static,
    {
        let resolvers = Self::get_middleware().await;

        let arc_name = Arc::new(name.to_string());
        resolvers
            .next(move |mut resolver, next| {
                let cb = callback.clone();
                let name = arc_name.clone();
                Box::pin(async move {
                    if let Ok(config) = resolver
                        .context_ref()
                        .get_config::<CacheConfig>("session")
                        .await
                    {
                        if config.storage_ref() == *name.as_ref() {
                            resolver = (cb)(resolver).await;
                        }
                    }

                    if !resolver.has_provider() {
                        next.call(resolver).await
                    } else {
                        resolver
                    }
                })
            })
            .await;
    }

    async fn get_middleware() -> Arc<simple_middleware::Manager<Self, Self>> {
        if let Some(r) = busybody::helpers::service_container().get().await {
            r
        } else {
            let manager = simple_middleware::Manager::<Self, Self>::last(|resolver, _| {
                Box::pin(async move { resolver })
            })
            .await;
            busybody::helpers::service_container()
                .set(manager)
                .await
                .get()
                .await
                .unwrap() // should never failed as we just registered the instance
        }
    }
}
