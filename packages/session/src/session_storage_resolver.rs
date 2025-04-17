use std::sync::Arc;

use anyhow::anyhow;
use dirtybase_contract::{app_contract::Context, session_contract::SessionStorageProvider};

use crate::SessionConfig;

pub struct SessionStorageResolver {
    context: Context,
    config: SessionConfig,
}

impl SessionStorageResolver {
    pub fn new(context: Context, config: SessionConfig) -> Self {
        Self { context, config }
    }

    /// Returns a reference to the current application's context
    pub fn context_ref(&self) -> &Context {
        &self.context
    }

    pub fn config_ref(&self) -> &SessionConfig {
        &self.config
    }

    /// Returns the current application's context
    pub fn context(&self) -> Context {
        self.context.clone()
    }

    pub async fn get_provider(self) -> Result<SessionStorageProvider, anyhow::Error> {
        let storage = Self::get_middleware().await.send(self).await?;
        Ok(SessionStorageProvider::from(storage))
    }

    pub async fn register<F, Fut>(name: &str, callback: F)
    where
        F: Clone + Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<SessionStorageProvider, anyhow::Error>> + Send + 'static,
    {
        let resolvers = Self::get_middleware().await;

        let arc_name = Arc::new(name.to_string());
        resolvers
            .next(move |resolver, next| {
                let cb = callback.clone();
                let name = arc_name.clone();
                Box::pin(async move {
                    if resolver.config_ref().storage_ref() == *name.as_ref() {
                        return (cb)(resolver).await;
                    }

                    next.call(resolver).await
                })
            })
            .await;
    }

    async fn get_middleware()
    -> Arc<simple_middleware::Manager<Self, Result<SessionStorageProvider, anyhow::Error>>> {
        if let Some(r) = busybody::helpers::service_container().get().await {
            r
        } else {
            let manager = simple_middleware::Manager::<
                Self,
                Result<SessionStorageProvider, anyhow::Error>,
            >::last(|resolver, _| {
                Box::pin(async move {
                    Err(anyhow!(
                        "could not get storage provider for: {}",
                        resolver.config_ref().storage_ref()
                    ))
                })
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
