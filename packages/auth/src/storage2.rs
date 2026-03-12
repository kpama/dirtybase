use dirtybase_common::db::base::manager::Manager;
use dirtybase_contract::{
    auth_contract::{
        Actor, PermissionManager, PermissionRepo,
        storage2::{PermStorageProvider, PermissionStorageResolver},
    },
    prelude::{ContextResourceManager, ResourceManager},
};

use crate::database_storage2::PermissionDatabaseStorage;

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

pub(crate) async fn register_manager() {
    ContextResourceManager::<PermissionManager>::register(
        |_| async move { Ok(("permission-manager", -1).into()) },
        |ctx| async move {
            let actor = ctx.get::<Actor>().await;
            eprintln!(
                "getting actor from context was sucessful? {}",
                actor.is_ok()
            );

            if let Ok(manager) = ctx.get::<Manager>().await {
                let mut permission_repo = PermissionRepo::new(&manager);

                if let Ok(a_permission) = permission_repo.get().await {
                    return Ok(PermissionManager::from(a_permission.iter()));
                }
            }
            Ok(PermissionManager::default())
        },
        |_| async move {
            // nothing to clean up, let it fall through
        },
    )
    .await;
}
