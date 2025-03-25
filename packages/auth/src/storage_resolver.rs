use std::{future::Future, sync::Arc};

use dirtybase_contract::{
    app::Context,
    auth::{AuthUserStorage, AuthUserStorageProvider},
    fama::{PipeContent, PipelineBuilder, PipelineBuilderTrait},
};

use crate::AuthConfig;

#[derive(Clone)]
pub struct StorageResolver {
    context: Context,
    provider: Option<Arc<AuthUserStorageProvider>>,
}

impl StorageResolver {
    pub fn new(context: Context) -> Self {
        Self {
            provider: None,
            context,
        }
    }

    pub fn has_provider(&self) -> bool {
        self.provider.is_some()
    }

    pub fn context_ref(&self) -> &Context {
        &self.context
    }

    pub fn context(&self) -> Context {
        self.context.clone()
    }

    pub fn set_storage(&mut self, storage: impl AuthUserStorage + 'static) {
        self.provider = Some(Arc::new(AuthUserStorageProvider::new(storage)));
    }

    pub async fn get_provider(self) -> Option<Arc<AuthUserStorageProvider>> {
        self.pipeline().await.deliver().await.provider
    }

    pub async fn provider<F, Fut>(callback: F)
    where
        F: Clone + Fn(Self, Arc<String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Self> + Send + 'static,
    {
        Self::pipeline_builder()
            .await
            .register(move |pipe| {
                let cb = callback.clone();
                Box::pin(async move {
                    let cb = cb.clone();
                    pipe.store_fn(cb)
                        .await
                        .next_fn(|p: Self| async move { !p.has_provider() }) // once we have a provider stop the pipe
                        .await
                })
            })
            .await;
    }
}

#[dirtybase_contract::async_trait]
impl PipelineBuilderTrait for StorageResolver {
    async fn setup_pipeline_builder(builder: PipelineBuilder<Self>) -> PipelineBuilder<Self> {
        builder
            .register(move |pipe| {
                Box::pin(async move {
                    pipe.next_fn(|pipe: PipeContent, p: Self| async move {
                        if let Ok(config) = p.context_ref().get::<AuthConfig>().await {
                            // storage name
                            pipe.store(config.storage()).await;
                            pipe.store(config).await;
                            return true;
                        }
                        return false;
                    })
                    .await
                })
            })
            .await;
        builder
    }
}
