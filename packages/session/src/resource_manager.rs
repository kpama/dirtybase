use dirtybase_contract::{
    app_contract::ContextResourceManager,
    session_contract::{SessionStorage, SessionStorageProvider},
};

use crate::{
    SessionConfig, SessionStorageResolver,
    storage::{database::DatabaseStorage, dummy::DummyStorage, memory::MemoryStorage},
};

pub async fn register_resource_manager() {
    // register resolver for the various storage providers
    register_storages().await;

    ContextResourceManager::<SessionStorageProvider>::register(
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
                let duration = if context.is_global() { 0 } else { 5 };
                context.set(config).await;
                (name, duration).into()
            })
        },
        |context| {
            //
            Box::pin(async move {
                let config = context
                    .get_config::<SessionConfig>("session")
                    .await
                    .unwrap();
                let provider = SessionStorageResolver::new(context.clone(), config)
                    .get_provider()
                    .await?;
                let storage = provider.clone();
                if let Ok(config) = context.get_config::<SessionConfig>("session").await {
                    let lifetime = config.lifetime();
                    let id = "session::storage".into();
                    let _ctx = dirtybase_cron::CronJob::schedule(
                        "every 25 minutes",
                        move |_| {
                            Box::pin({
                                let storage = storage.clone();
                                async move {
                                    storage.gc(lifetime).await;
                                }
                            })
                        },
                        id,
                    )
                    .await;
                }

                Ok(provider)
            })
        },
        |_provider| {
            //
            Box::pin(async {
                // TODO: Close the storage driver
            })
        },
    )
    .await;
}

async fn register_storages() {
    DatabaseStorage::register().await;
    DummyStorage::register().await;
    MemoryStorage::register().await;
}
