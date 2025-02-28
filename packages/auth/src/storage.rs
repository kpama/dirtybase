mod database_storage;
mod memory_storage;

pub use database_storage::*;
use dirtybase_contract::app::ContextManager;
use dirtybase_contract::auth::AuthService;
pub use dirtybase_contract::fama::PipeContent;
pub use dirtybase_contract::fama::PipelineBuilderTrait;
pub use memory_storage::*;

use crate::AuthConfig;
use crate::AuthUserStorageDriver;

pub(crate) async fn register_storages() {
    dirtybase_contract::auth::StorageResolverPipeline::pipeline_builder()
        .await
        .register(move |pipe| {
            Box::pin(async move {
                pipe.next_fn(
                    async |mut p: dirtybase_contract::auth::StorageResolverPipeline,
                           c: PipeContent| {
                        let config = c.container().get_type::<AuthConfig>().await.unwrap();

                        tracing::warn!(" we got call to when resolving auth user stoarge");
                        tracing::warn!("storage already exist?: {}", p.has_provider());
                        tracing::warn!("context id: {}", p.context_ref().id_ref());
                        tracing::warn!("config: {:#?}", &config);

                        if !p.has_provider() {
                            match config.storage_ref() {
                                AuthUserStorageDriver::Memory => {
                                    p.set_storage(AuthUserMemoryStorage::new());
                                    c.store(p).await;
                                }
                                AuthUserStorageDriver::Database => {
                                    tracing::error!("setup database storage driver for auth user");
                                }
                                AuthUserStorageDriver::Custom(_) => {
                                    tracing::info!("custom driver ");
                                }
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
    busybody::helpers::service_container()
        .set(ContextManager::<AuthService>::new())
        .await;
}
