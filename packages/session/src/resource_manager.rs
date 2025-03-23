use std::sync::Arc;

use anyhow::anyhow;
use dirtybase_contract::{
    app::ContextResourceManager,
    session::{SessionStorage, SessionStorageProvider, SessionStorageProviderService},
};
use dirtybase_cron::prelude::{DispatchableQuery, DispatchedQuery, QueryHandler};

use crate::{
    MakeSessionStorageCommand, MakeSessionStorageResult, SessionConfig,
    storage::{DummyStorage, MemoryStorage},
};

#[derive(Default)]
struct MakeSessionStorageCommandHandler;

#[async_trait::async_trait]
impl QueryHandler for MakeSessionStorageCommandHandler {
    async fn handle_query(&self, mut dispatched: DispatchedQuery) -> DispatchedQuery {
        if let Some(query) = dispatched.the_query_mut::<MakeSessionStorageCommand>() {
            let lifetime = query.config_ref().lifetime();
            match query.config_ref().storage_ref() {
                "dummy" => {
                    let provider = Arc::new(SessionStorageProvider::from(DummyStorage::default()));
                    dispatched.set_value::<MakeSessionStorageResult>(Ok(provider));
                }
                "memory" => {
                    let provider = MemoryStorage::make_provider().await;
                    let storage = provider.clone();
                    let _ctx = dirtybase_cron::CronJob::register(
                        "every 5 minutes",
                        move |_| {
                            Box::pin({
                                let storage = storage.clone();
                                async move {
                                    storage.gc(lifetime).await;
                                }
                            })
                        },
                        "session::memory-storage",
                    )
                    .await;
                    dispatched.set_value::<MakeSessionStorageResult>(Ok(provider));
                }
                "database" => dispatched
                    .set_value::<MakeSessionStorageResult>(Err(anyhow!("not implemented yet"))),
                "file" => dispatched
                    .set_value::<MakeSessionStorageResult>(Err(anyhow!("not implemented yet"))),
                "redis" => dispatched
                    .set_value::<MakeSessionStorageResult>(Err(anyhow!("not implemented yet"))),
                _ => (),
            }
        }
        dispatched
    }
}

pub(crate) async fn register_resource_manager() {
    MakeSessionStorageCommand::query_handler::<MakeSessionStorageCommandHandler>().await;

    ContextResourceManager::<SessionStorageProviderService>::register(
        |context| {
            //
            Box::pin(async move {
                let config = context
                    .get_config::<SessionConfig>("session")
                    .await
                    .unwrap();
                let name = context
                    .tenant()
                    .await
                    .expect("could not get tenant")
                    .id()
                    .to_string();
                let duration = if config.storage_ref() == "memory" {
                    0
                } else {
                    0 // FIXME: Use the right duration time
                };
                context.set(config).await;
                (name, duration)
            })
        },
        |context| {
            //
            Box::pin(async move {
                let _config = context.get::<SessionConfig>().await.unwrap();
                let config = context
                    .get_config::<SessionConfig>("session")
                    .await
                    .unwrap();
                if let Some(result) = MakeSessionStorageCommand::new(config)
                    .dispatch_query()
                    .await
                    .take_value::<MakeSessionStorageResult>()
                {
                    *result
                } else {
                    Err(anyhow!("could not...."))
                }
            })
        },
        |context| {
            //
            Box::pin(async {
                // TODO: Close the storage driver
            })
        },
    )
    .await;
}
