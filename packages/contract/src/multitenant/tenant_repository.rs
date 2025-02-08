use std::{ops::Deref, sync::Arc};

use crate::db::types::ArcUuid7;

use super::{DummyTenantRepository, TenantContext};

#[async_trait::async_trait]
pub trait TenantRepositoryTrait: Send + Sync {
    /// Find the current conte
    async fn by_id(&self, id: ArcUuid7) -> Option<TenantContext>;
}

#[derive(Clone)]
pub struct TenantRepositoryProvider(Arc<Box<dyn TenantRepositoryTrait>>);

#[async_trait::async_trait]
impl TenantRepositoryTrait for TenantRepositoryProvider {
    async fn by_id(&self, id: ArcUuid7) -> Option<TenantContext> {
        self.0.by_id(id).await
    }
}

impl TenantRepositoryProvider {
    pub fn from<T: TenantRepositoryTrait + 'static>(manager: T) -> Self {
        Self(Arc::new(Box::new(manager)))
    }
}

impl Deref for TenantRepositoryProvider {
    type Target = Arc<Box<dyn TenantRepositoryTrait>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for TenantRepositoryProvider {
    fn default() -> Self {
        Self(Arc::new(Box::new(DummyTenantRepository)))
    }
}
