use anyhow::Context as AnyhowCtx;
use dirtybase_contract::{
    app_contract::ContextResourceManager,
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
        |context| {
            //
            Box::pin(async move {
                let config = SessionExtension::config_from_ctx(&context)
                    .await
                    .unwrap_or_default();
                let name = context
                    .tenant_context()
                    .await
                    .context("could not get tenant context")?
                    .id()
                    .to_string();
                let duration = if context.is_global() { 0 } else { 5 };
                context.set(config).await;
                Ok((name, duration).into())
            })
        },
        |context| {
            //
            Box::pin(async move {
                let config = context.get::<SessionConfig>().await.unwrap_or_default();
                let lifetime = config.lifetime();
                let provider = SessionStorageResolver::new(context.clone())
                    .get_provider(config.storage().to_string())
                    .await?;
                let storage = provider.clone();
                let id = "session::storage".into();
                let ctx = dirtybase_cron::CronJob::schedule(
                    "every 25 minutes",
                    move |_| {
                        Box::pin({
                            let storage = storage.clone();
                            async move {
                                storage.gc(lifetime).await;
                                tracing::trace!("session gc executed");
                            }
                        })
                    },
                    id,
                )
                .await;
                tracing::trace!("session gc scheduled: {}", ctx.is_ok());

                Ok(provider)
            })
        },
        |_provider| {
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
