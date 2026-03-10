pub mod database_storage;
pub mod database_storage2;
pub mod memory_storage;

use anyhow::Context;
use dirtybase_contract::auth_contract::AuthUserStorageProvider;
use dirtybase_contract::auth_contract::StorageResolver;
pub use dirtybase_contract::fama::PipeContent;
pub use dirtybase_contract::fama::PipelineBuilderTrait;
use dirtybase_contract::prelude::ContextResourceManager;
use dirtybase_contract::prelude::ResourceManager;

use crate::AuthConfig;
use crate::database_storage::AuthUserDatabaseStorage;
use crate::memory_storage::AuthUserMemoryStorage;

pub(crate) async fn register_storages() {
    // database storage
    AuthUserDatabaseStorage::register().await;

    // memory storage
    AuthUserMemoryStorage::register().await;

    ContextResourceManager::<AuthUserStorageProvider>::register(
        |ctx| async move {
            _ = ctx.load_config::<AuthConfig>("auth").await?;
            let name = ctx
                .tenant_context()
                .await
                .context("coult not get tenant context")?
                .id()
                .to_string();
            Ok(ResourceManager::scoped(&name))
        },
        |ctx| async move {
            let config = ctx.get::<AuthConfig>().await?;
            let resolver = StorageResolver::new(ctx.clone());

            tracing::trace!("resolving auth auther storage: {}", config.storage());

            resolver
                .get_provider(&config.storage())
                .await
                .ok_or(anyhow::anyhow!("could not resolve storage"))
        },
        |_| async {
            // nothign to do here at the moment
        },
    )
    .await;
}
