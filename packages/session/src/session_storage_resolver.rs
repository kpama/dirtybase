use std::sync::Arc;

use anyhow::anyhow;
use dirtybase_contract::{
    app_contract::Context,
    session_contract::{SessionStorage, SessionStorageProvider},
};

use crate::SessionConfig;

pub struct SessionStorageResolver {
    context: Context,
    provider: Option<SessionStorageProvider>,
    config: SessionConfig,
}

impl SessionStorageResolver {
    pub fn new(context: Context, config: SessionConfig) -> Self {
        Self {
            provider: None,
            context,
            config,
        }
    }

    /// Checks if a provider has been set
    pub fn has_provider(&self) -> bool {
        self.provider.is_some()
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

    pub fn set_storage(&mut self, storage: impl SessionStorage + 'static) {
        self.provider = Some(SessionStorageProvider::from(storage));
    }

    pub async fn get_provider(self) -> Result<SessionStorageProvider, anyhow::Error> {
        match Self::get_middleware().await.send(self).await.provider {
            Some(p) => Ok(p),
            None => Err(anyhow!("could not resolve the session storage provider")),
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
                    if resolver.config_ref().storage_ref() == *name.as_ref() {
                        resolver = (cb)(resolver).await;
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
