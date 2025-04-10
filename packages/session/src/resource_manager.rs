use dirtybase_contract::{
    app_contract::ContextResourceManager, session_contract::SessionStorageProvider,
};

use crate::{SessionConfig, SessionStorageResolver, storage};

pub(crate) async fn register_resource_manager() {
    // register resolver for the various storage providers
    SessionStorageResolver::register(storage::database::NAME, storage::database::resolver).await;
    SessionStorageResolver::register(storage::dummy::NAME, storage::dummy::resolver).await;
    SessionStorageResolver::register(storage::memory::NAME, storage::memory::resolver).await;

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
                let config = context
                    .get_config::<SessionConfig>("session")
                    .await
                    .unwrap();
                SessionStorageResolver::new(context, config)
                    .get_provider()
                    .await
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
