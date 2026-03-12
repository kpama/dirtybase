use dirtybase_contract::{
    app_contract::ContextResourceManager,
    prelude::ResourceManager,
    session_contract::{SessionStorage, SessionStorageProvider, SessionStorageResolver},
};

use crate::{
    SessionConfig, SessionExtension,
    storage::{database::DatabaseStorage, dummy::DummyStorage, memory::MemoryStorage},
};

pub async fn register_resource_manager() {
    // Register resolver for the various storage providers
    register_storages().await;

    ContextResourceManager::<SessionStorageProvider>::register(
        |context| async move {
            let config = SessionExtension::config_from_ctx(&context)
                .await
                .unwrap_or_default();

            context.set(config).await;
            Ok(ResourceManager::scoped("session"))
        },
        |context| async move {
            let config = context.get::<SessionConfig>().await.unwrap_or_default();
            let lifetime = config.lifetime();
            let provider = SessionStorageResolver::new(context.clone())
                .get_provider(config.storage().to_string())
                .await?;
            let storage = provider.clone();

            let lottery = config.lottery();
            tokio::task::spawn(async move {
                let selection = rand::random_range(1..=lottery[1]);
                if selection <= lottery[0] {
                    storage.gc(lifetime).await;
                }
            });

            Ok(provider)
        },
        |_provider| {
            async {
                // TODO: Close the storage driver
            }
        },
    )
    .await;
}

async fn register_storages() {
    DatabaseStorage::register().await;
    DummyStorage::register().await;
    MemoryStorage::register().await;
}
