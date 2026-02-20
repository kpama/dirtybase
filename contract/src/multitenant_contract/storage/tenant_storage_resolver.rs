use std::{future::Future, sync::Arc};

use simple_middleware::Manager;

use crate::{multitenant_contract::TenantStorageProvider, prelude::Context};

#[derive(Clone)]
pub struct TenantStorageResolver {
    context: Context,
}

impl TenantStorageResolver {
    /// Create a new `Resolver` instance
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    /// Returns a reference to the context
    pub fn context(&self) -> &Context {
        &self.context
    }

    pub async fn get_provider(self, name: String) -> Result<TenantStorageProvider, anyhow::Error> {
        Self::get_middleware().await.send((self, name)).await
    }

    pub async fn register<F, Fut>(name: &str, callback: F)
    where
        F: Clone + Fn(Self) -> Fut + Send + 'static,
        Fut: Future<Output = Result<TenantStorageProvider, anyhow::Error>> + Send + 'static,
    {
        let arc_name = Arc::new(name.to_string());
        Self::get_middleware()
            .await
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

    async fn get_middleware()
    -> Arc<Manager<(Self, String), Result<TenantStorageProvider, anyhow::Error>>> {
        if let Some(m) = busybody::helpers::get_service().await {
            m
        } else {
            let manager = simple_middleware::Manager::<
                (Self, String),
                Result<TenantStorageProvider, anyhow::Error>,
            >::last(|(_, name), _| async move {
                Err(anyhow::anyhow!(
                    "no register tenant storage provider for: {}",
                    name
                ))
            })
            .await;

            busybody::helpers::service_container()
                .set(manager)
                .await
                .get()
                .await
                .unwrap() // Should never fail as we just registered the instance
        }
    }
}
