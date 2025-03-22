use std::sync::Arc;

use anyhow::anyhow;
use dirtybase_contract::{
    app::ContextResourceManager,
    session::{SessionStorage, SessionStorageProvider, SessionStorageProviderService},
};

use crate::{
    SessionConfig,
    storage::{DummyStorage, MemoryStorage},
};

pub(crate) async fn register_resource_manager() {
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
                let lifetime = config.lifetime();
                match config.storage_ref() {
                    "dummy" => Ok(Arc::new(SessionStorageProvider::from(
                        DummyStorage::default(),
                    ))),
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
                        Ok(provider)
                    }
                    "database" => Err(anyhow!("not implemented yet")),
                    "file" => Err(anyhow!("not implemented yet")),
                    "redis" => Err(anyhow!("not implemented yet")),
                    _ => Err(anyhow!("not implemented yet")),
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
