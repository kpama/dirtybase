use anyhow::Context as AnyhowCtx;
use dirtybase_common::db::base::manager::Manager;
use dirtybase_contract::{
    multitenant_contract::{TenantStorageProvider, TenantStorageResolver, model::TenantRepo},
    prelude::{ContextResourceManager, ResourceManager},
};

use crate::{MultiTenantManager, MultitenantConfig, storage::TenantDatabaseStorage};

pub(crate) async fn register_multitenant_resource_manager() {
    register_storages().await;

    ContextResourceManager::<MultiTenantManager>::register(
        |_| async move { Ok(ResourceManager::forever("multitenant-manager")) },
        |context| async move {
            let config = context
                .get::<MultitenantConfig>()
                .await
                .context("could not get multitenant config")?;

            let storage = TenantStorageResolver::new(context.clone())
                .get_provider(config.storage().to_string())
                .await
                .context("could not get tenant storage provider")?;
            Ok(MultiTenantManager::new(config, storage))
        },
        |_| async {
            // NOTE: We will never drop unless the program has ended
        },
    )
    .await;
}

async fn register_storages() {
    let handler = |resolver: TenantStorageResolver| async move {
        let manager = resolver
            .context()
            .get::<Manager>()
            .await
            .context("could not get database manager when setting up tenant storage")?;

        Ok(TenantStorageProvider::new(TenantDatabaseStorage::new(
            TenantRepo::new(&manager),
        )))
    };
    TenantStorageResolver::register("db", handler).await;
    TenantStorageResolver::register("database", handler).await;
}
