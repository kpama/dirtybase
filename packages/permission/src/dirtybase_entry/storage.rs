mod database_storage;

use dirtybase_common::db::base::manager::Manager;
use dirtybase_contract::prelude::{
    ContextResourceManager, PermStorageProvider, PermissionStorageResolver, ResourceManager,
};

use crate::dirtybase_entry::storage::database_storage::PermissionDatabaseStorage;

pub async fn register_storage() {
    ContextResourceManager::<PermStorageProvider>::register(
        |_| async move {
            // FIXME: Should be based on the current tenant
            Ok(ResourceManager::scoped("permission"))
        },
        |c| async move {
            // FIXME: Implement configuration for the permission crate
            let name = "database";
            PermissionStorageResolver::new(c)
                .get_provider(name.to_string())
                .await
        },
        |_| async move {
            //
        },
    )
    .await;

    PermissionStorageResolver::register("database", |resolver| async move {
        let manager = resolver
            .context()
            .get::<Manager>()
            .await
            .expect("could not get database manager for permission storage");
        Ok(PermStorageProvider::new(PermissionDatabaseStorage::new(
            manager,
        )))
    })
    .await;
}
