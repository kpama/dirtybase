use std::{future::Future, sync::Arc};

use dirtybase_contract::{
    app_contract::Context,
    auth_contract::{AuthUserStorage, AuthUserStorageProvider},
};

use crate::AuthConfig;

#[derive(Clone)]
pub struct StorageResolver {
    context: Context,
    name: Arc<String>,
    provider: Option<AuthUserStorageProvider>,
}

impl StorageResolver {
    /// Creates a new instance of this struct
    pub fn new(context: Context, name: &str) -> Self {
        Self {
            provider: None,
            name: Arc::new(name.to_string()),
            context,
        }
    }

    pub async fn from_context(context: Context) -> Self {
        Self {
            provider: None,
            name: context
                .get_config::<AuthConfig>("auth")
                .await
                .unwrap()
                .storage(),
            context,
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

    /// Returns the current application's context
    pub fn context(&self) -> Context {
        self.context.clone()
    }

    /// Sets the storage instance
    pub fn set_storage(&mut self, storage: impl AuthUserStorage + 'static) {
        self.provider = Some(AuthUserStorageProvider::new(storage));
    }

    pub async fn get_provider(self) -> Option<AuthUserStorageProvider> {
        Self::get_middleware().await.send(self).await.provider
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
                    if resolver.name_rf() == *name.as_ref() {
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

    fn name_rf(&self) -> &str {
        &self.name
    }
}
