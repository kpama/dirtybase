mod database_storage;
mod memory_storage;

pub use database_storage::*;
use dirtybase_contract::app::ContextResourceManager;
use dirtybase_contract::db::base::manager::Manager;
pub use dirtybase_contract::fama::PipeContent;
pub use dirtybase_contract::fama::PipelineBuilderTrait;
pub use memory_storage::*;

use crate::AuthConfig;
use crate::DATABASE_STORAGE;
use crate::MEMORY_STORAGE;

pub(crate) async fn register_storages() {
    dirtybase_contract::auth::StorageResolverPipeline::pipeline_builder()
        .await
        .register(move |pipe| {
            Box::pin(async move {
                pipe.next_fn(
                    async |mut p: dirtybase_contract::auth::StorageResolverPipeline,
                           c: PipeContent| {
                        let config = p.context_ref().get::<AuthConfig>().await.unwrap();

                        tracing::warn!("we got call to when resolving auth user storage");
                        tracing::warn!("storage already exist?: {}", p.has_provider());
                        tracing::warn!("context id: {}", p.context_ref().id_ref());
                        tracing::warn!("config: {:#?}", &config);

                        if !p.has_provider() {
                            match config.storage_ref().as_str() {
                                MEMORY_STORAGE => {
                                    p.set_storage(AuthUserMemoryStorage::new());
                                    c.store(p).await;
                                }
                                DATABASE_STORAGE => {
                                    tracing::error!("Auth database driver setting up");
                                    let manager =
                                    //     ContextResourceManager::<Manager>::try_get(p.context_ref())
                                    //         .await
                                    //         .unwrap();
                                    p.context_ref().get::<Manager>().await.unwrap();
                                    let result = manager.has_table("auth_users").await;
                                    tracing::error!("do we have the auth_users table: {}", result);

                                    p.set_storage(AuthUserDatabaseStorage::new(manager));
                                    c.store(p).await;
                                    tracing::debug!(
                                        "database driver configured as auth user provider"
                                    );
                                }
                                _ => (),
                            }
                        }
                        true
                    },
                )
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
