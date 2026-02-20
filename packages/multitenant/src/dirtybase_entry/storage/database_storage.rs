use tokio::sync::RwLock;

use dirtybase_common::anyhow::{self};
use dirtybase_contract::{
    async_trait,
    multitenant_contract::{
        TenantStorage,
        model::{FetchTenantOption, FetchTenantPayload, Tenant, TenantId, TenantRepo},
    },
};

pub struct TenantDatabaseStorage {
    repo: RwLock<TenantRepo>,
}

impl TenantDatabaseStorage {
    pub fn new(repo: TenantRepo) -> Self {
        Self {
            repo: RwLock::new(repo),
        }
    }
}

#[async_trait]
impl TenantStorage for TenantDatabaseStorage {
    async fn save(&self, tenant: Tenant) -> Result<Tenant, anyhow::Error> {
        let mut w_lock = self.repo.write().await;
        if tenant.created_at().is_some() {
            w_lock.update(tenant).await
        } else {
            w_lock.insert(tenant).await
        }
    }

    async fn delete(&self, id: TenantId) -> Result<(), anyhow::Error> {
        let mut w_lock = self.repo.write().await;
        w_lock.delete_by_id(id).await
    }

    async fn restore(&self, id: TenantId) -> Result<(), anyhow::Error> {
        let mut w_lock = self.repo.write().await;
        _ = w_lock.restore(id).await?;
        Ok(())
    }

    async fn destroy(&self, id: TenantId) -> Result<(), anyhow::Error> {
        let mut w_lock = self.repo.write().await;
        w_lock.destroy_by_id(id).await
    }

    async fn find(
        &self,
        payload: FetchTenantPayload,
        _option: Option<FetchTenantOption>,
    ) -> Result<Option<Tenant>, anyhow::Error> {
        let mut w_lock = self.repo.write().await;
        match payload {
            FetchTenantPayload::ById { id } => w_lock.by_id(id).await,
            FetchTenantPayload::ByToken { token } => {
                w_lock
                    .filter(move |builder| {
                        builder.is_eq(Tenant::col_name_for_token(), token);
                    })
                    .one()
                    .await
            }
            FetchTenantPayload::ByDomain { domain } => {
                w_lock
                    .filter(|builder| {
                        builder.is_eq(Tenant::col_name_for_domain(), domain);
                    })
                    .one()
                    .await
            }
            FetchTenantPayload::ByName { name } => {
                w_lock
                    .filter(|builder| {
                        builder.is_eq(Tenant::col_name_for_name(), name.clone());
                    })
                    .one()
                    .await
            }
        }
    }
}
