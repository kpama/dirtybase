use std::sync::Arc;

use fama::PipelineBuilderTrait;

use crate::app::Context;

use super::{AuthUserStorage, AuthUserStorageProvider};

#[derive(Clone)]
pub struct StorageResolverPipeline {
    context: Context,
    provider: Option<Arc<AuthUserStorageProvider>>,
}

impl StorageResolverPipeline {
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
        let x = self.pipeline().await.deliver().await;

        x.provider
        // .expect("no auth user storage provider found")
    }
}

#[async_trait::async_trait]
impl PipelineBuilderTrait for StorageResolverPipeline {}
