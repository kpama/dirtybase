use std::{ops::Deref, sync::Arc};

use crate::db_contract::types::ArcUuid7;

use super::TenantContext;

#[async_trait::async_trait]
pub trait TenantStorageTrait: Send + Sync {
    /// Find the current conte
    async fn by_id(&self, id: ArcUuid7) -> Option<TenantContext>;
}

#[derive(Clone)]
pub struct TenantStorageProvider(Arc<Box<dyn TenantStorageTrait>>);

#[async_trait::async_trait]
impl TenantStorageTrait for TenantStorageProvider {
    async fn by_id(&self, id: ArcUuid7) -> Option<TenantContext> {
        self.0.by_id(id).await
    }
}

impl TenantStorageProvider {
    pub fn from<T: TenantStorageTrait + 'static>(manager: T) -> Self {
        Self(Arc::new(Box::new(manager)))
    }
}

impl Deref for TenantStorageProvider {
    type Target = Arc<Box<dyn TenantStorageTrait>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
