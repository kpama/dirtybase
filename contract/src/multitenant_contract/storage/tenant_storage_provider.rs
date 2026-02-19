use std::sync::Arc;

use crate::multitenant_contract::model::{FetchTenantOption, FetchTenantPayload, Tenant, TenantId};

#[async_trait::async_trait]
pub trait TenantStorage: Send + Sync {
    /// Saves a Tenant and returns the saved instance
    async fn save(&self, tenant: Tenant) -> Result<Tenant, anyhow::Error>;

    /// Soft delete the Tenant with the specified ID
    async fn delete(&self, id: TenantId) -> Result<(), anyhow::Error>;

    /// Restores the tenant with the specified ID that was previously soft deleted
    async fn restore(&self, id: TenantId) -> Result<(), anyhow::Error>;

    /// Permanently delete the Tenant with the specified ID
    async fn destroy(&self, id: TenantId) -> Result<(), anyhow::Error>;

    // Fetches a Tenant
    async fn find(
        &self,
        payload: FetchTenantPayload,
        option: Option<FetchTenantOption>,
    ) -> Result<Option<Tenant>, anyhow::Error>;
}

#[derive(Clone)]
pub struct TenantStorageProvider(Arc<Box<dyn TenantStorage>>);

impl TenantStorageProvider {
    pub fn new(storage: impl TenantStorage + 'static) -> Self {
        Self(Arc::new(Box::new(storage)))
    }
}

#[async_trait::async_trait]
impl TenantStorage for TenantStorageProvider {
    async fn save(&self, tenant: Tenant) -> Result<Tenant, anyhow::Error> {
        self.0.save(tenant).await
    }

    async fn delete(&self, id: TenantId) -> Result<(), anyhow::Error> {
        self.0.delete(id).await
    }

    async fn restore(&self, id: TenantId) -> Result<(), anyhow::Error> {
        self.0.restore(id).await
    }

    async fn destroy(&self, id: TenantId) -> Result<(), anyhow::Error> {
        self.0.destroy(id).await
    }

    async fn find(
        &self,
        payload: FetchTenantPayload,
        option: Option<FetchTenantOption>,
    ) -> Result<Option<Tenant>, anyhow::Error> {
        self.0.find(payload, option).await
    }
}
