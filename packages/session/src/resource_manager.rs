use dirtybase_contract::{
    app_contract::ContextResourceManager, session_contract::SessionStorageProvider,
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
                (name, duration)
            })
        },
        |context| {
            //
            Box::pin(async move {
                let config = context
                    .get_config::<SessionConfig>("session")
                    .await
                    .unwrap();
                SessionStorageResolver::new(context, config)
                    .get_provider()
                    .await
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
