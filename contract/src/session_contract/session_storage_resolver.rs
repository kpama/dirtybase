use std::{future::Future, sync::Arc};

use anyhow::anyhow;

use crate::{prelude::Context, session_contract::SessionStorageProvider};

pub struct SessionStorageResolver {
    context: Context,
}

impl SessionStorageResolver {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    /// Returns a reference to the current application's context
    pub fn context_ref(&self) -> &Context {
        &self.context
    }

    /// Returns the current application's context
    pub fn context(&self) -> Context {
        self.context.clone()
    }

    pub async fn get_provider(self, name: String) -> Result<SessionStorageProvider, anyhow::Error> {
        let storage = Self::get_middleware().await.send((self, name)).await?;
        Ok(storage)
    }

    pub async fn register<F, Fut>(name: &str, callback: F)
    where
        F: Clone + Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<SessionStorageProvider, anyhow::Error>> + Send + 'static,
    {
        let resolvers = Self::get_middleware().await;

        let arc_name = Arc::new(name.to_string());
        resolvers
            .next(move |(resolver, n), next| {
                let cb = callback.clone();
                let name = arc_name.clone();
                async move {
                    if n == *name.as_ref() {
                        return (cb)(resolver).await;
                    }

                    next.call((resolver, n)).await
                }
            })
            .await;
    }

    async fn get_middleware() -> Arc<
        simple_middleware::Manager<(Self, String), Result<SessionStorageProvider, anyhow::Error>>,
    > {
        if let Some(r) = busybody::helpers::service_container().get().await {
            r
        } else {
            let manager = simple_middleware::Manager::<
                (Self, String),
                Result<SessionStorageProvider, anyhow::Error>,
            >::last(|(_, name), _| async move {
                Err(anyhow!("could not get storage provider for: {}", name))
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
