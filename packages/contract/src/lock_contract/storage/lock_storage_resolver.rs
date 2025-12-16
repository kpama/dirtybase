use std::{future::Future, sync::Arc};

use crate::{lock_contract::storage::LockStorageProvider, prelude::Context};

#[derive(Clone)]
pub struct StorageResolver {
    context: Context,
}

impl StorageResolver {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn from_context(context: Context) -> Self {
        Self::new(context)
    }

    pub fn context_ref(&self) -> &Context {
        &self.context
    }

    pub fn context(&self) -> Context {
        self.context.clone()
    }

    pub async fn get_provider(self, name: String) -> Result<LockStorageProvider, anyhow::Error> {
        Self::get_middleware().await.send((self, name)).await
    }

    pub async fn register<F, Fut>(name: &str, callback: F)
    where
        F: Clone + Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<LockStorageProvider, anyhow::Error>> + Send + 'static,
    {
        let resolvers = Self::get_middleware().await;
        let arc_name = Arc::new(name.to_string());
        resolvers
            .next(move |(resolver, name), next| {
                let cb = callback.clone();
                let n = arc_name.clone();
                async move {
                    if name == *n {
                        return (cb)(resolver).await;
                    }
                    next.call((resolver, name)).await
                }
            })
            .await;
    }

    async fn get_middleware(
    ) -> Arc<simple_middleware::Manager<(Self, String), Result<LockStorageProvider, anyhow::Error>>>
    {
        if let Some(m) = busybody::helpers::service_container().get().await {
            m
        } else {
            let manager = simple_middleware::Manager::<
                (Self, String),
                Result<LockStorageProvider, anyhow::Error>,
            >::last(|(_, name), _| async move {
                Err(anyhow::anyhow!(
                    "no register lock storage provider for: {}",
                    name
                ))
            })
            .await;

            busybody::helpers::service_container()
                .set(manager)
                .await
                .get()
                .await
                .unwrap() // Should never failed as we just registered the instance
        }
    }
}
