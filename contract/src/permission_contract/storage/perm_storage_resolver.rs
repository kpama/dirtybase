use std::{future::Future, sync::Arc};

use crate::prelude::{Context, PermStorageProvider};

#[derive(Clone)]
pub struct PermissionStorageResolver {
    context: Context,
}

impl PermissionStorageResolver {
    /// Creates a new `Resolver` instance
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    /// Returns a reference to the context instance
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Tries to get the provider with the specified name
    pub async fn get_provider(self, name: String) -> Result<PermStorageProvider, anyhow::Error> {
        Self::get_middleware().await.send((self, name)).await
    }

    /// Registers a provider's resolver for the specified name
    pub async fn register<F, Fut>(name: &str, callback: F)
    where
        F: Clone + Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<PermStorageProvider, anyhow::Error>> + Send + 'static,
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
                    } else {
                        next.call((resolver, name)).await
                    }
                }
            })
            .await;
    }

    async fn get_middleware(
    ) -> Arc<simple_middleware::Manager<(Self, String), Result<PermStorageProvider, anyhow::Error>>>
    {
        if let Some(m) = busybody::helpers::get_service().await {
            m
        } else {
            let manager = simple_middleware::Manager::<
                (Self, String),
                Result<PermStorageProvider, anyhow::Error>,
            >::last(|(_, name), _| async move {
                Err(anyhow::anyhow!(
                    "no register permission storage provider for: {}",
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
