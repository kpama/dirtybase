mod database_storage;
mod memory_storage;

pub use database_storage::*;
use dirtybase_contract::db_contract::base::manager::Manager;
pub use dirtybase_contract::fama::PipeContent;
pub use dirtybase_contract::fama::PipelineBuilderTrait;
pub use memory_storage::*;

use crate::AUTH_USER_TABLE;
use crate::AuthConfig;
use crate::DATABASE_STORAGE;
use crate::MEMORY_STORAGE;
use crate::StorageResolver;

pub(crate) async fn register_storages() {
    // database storage
    StorageResolver::register(DATABASE_STORAGE, |mut resolver| async move {
        tracing::trace!("setting up database auth storage");
        // let config = resolver.context_ref().get::<AuthConfig>().await.unwrap();
        if let Ok(manager) = resolver.context_ref().get::<Manager>().await {
            resolver.set_storage(AuthUserDatabaseStorage::new(manager));
        }

        resolver
    })
    .await;

    // memory storage
    StorageResolver::register(MEMORY_STORAGE, |mut resolver| async move {
        tracing::trace!("setting up memory auth storage");
        resolver.set_storage(AuthUserMemoryStorage::new());

        resolver
    })
    .await;
}

pub(crate) async fn setup_context_managers() {
    // busybody::helpers::service_container()
    //     .set(ContextResourceManager::<AuthService>::new())
    //     .await;
}
