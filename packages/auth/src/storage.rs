mod database_storage;
mod memory_storage;

pub use database_storage::*;
use dirtybase_contract::db::base::manager::Manager;
pub use dirtybase_contract::fama::PipeContent;
pub use dirtybase_contract::fama::PipelineBuilderTrait;
pub use memory_storage::*;

use crate::AuthConfig;
use crate::DATABASE_STORAGE;
use crate::MEMORY_STORAGE;
use crate::StorageResolver;

pub(crate) async fn register_storages() {
    // database storage
    StorageResolver::provider(|p, name| async move {
        let config = p.context_ref().get::<AuthConfig>().await.unwrap();
        println!(">>>>>>>>>>>>>>>>>>>> name: {:?}", name);
        if name.as_str() == DATABASE_STORAGE {
            tracing::error!("cleanner implementation for database .....");
        }
        p
    })
    .await;

    // memory storage
    StorageResolver::provider(|p, name| async move {
        if name.as_str() == MEMORY_STORAGE {
            tracing::error!("cleanner implementation for memory.....");
        }
        p
    })
    .await;

    StorageResolver::pipeline_builder()
        .await
        .register(move |pipe| {
            Box::pin(async move {
                pipe.next_fn(async |mut p: StorageResolver, c: PipeContent| {
                    let config = p.context_ref().get::<AuthConfig>().await.unwrap();

                    if !p.has_provider() {
                        match config.storage_ref().as_str() {
                            MEMORY_STORAGE => {
                                p.set_storage(AuthUserMemoryStorage::new());
                                c.store(p).await;
                            }
                            DATABASE_STORAGE => {
                                tracing::error!(
                                    ">>>>>>>>>>>>>>>>>>>>>> Auth database driver setting up"
                                );
                                let manager = p.context_ref().get::<Manager>().await.unwrap();
                                let result = manager.has_table("auth_users").await;
                                tracing::error!("do we have the auth_users table: {}", result);

                                p.set_storage(AuthUserDatabaseStorage::new(manager));
                                c.store(p).await;
                                tracing::debug!("database driver configured as auth user provider");
                            }
                            _ => (),
                        }
                    }
                    true
                })
                .await
            })
        })
        .await;
}

pub(crate) async fn setup_context_managers() {
    // busybody::helpers::service_container()
    //     .set(ContextResourceManager::<AuthService>::new())
    //     .await;
}
