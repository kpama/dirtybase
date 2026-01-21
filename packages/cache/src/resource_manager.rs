use dirtybase_contract::prelude::ContextResourceManager;

use crate::{CacheManager, CacheStorageResolver, cache_store::MemoryStore, config::CacheConfig};

pub async fn register_resource_manager() {
    ContextResourceManager::<CacheManager>::register(
        |context| {
            //
            Box::pin(async move {
                //...
                // TODO: Source the idle timeout from config
                let name = if let Some(t) = context.tenant_context().await {
                    t.id_as_string()
                } else {
                    String::from("unknown")
                };
                Ok((name, 5).into())
            })
        },
        |context| {
            //
            Box::pin(async move {
                let config = context.get_config::<CacheConfig>("cache").await.unwrap();
                let _name = context
                    .tenant_context()
                    .await
                    .expect("could not get tenant")
                    .id()
                    .to_string();
                let provider = CacheStorageResolver::new(context, config)
                    .get_provider()
                    .await?;
                // TODO: fix prefix name
                Ok(CacheManager::new(provider, Some("core".to_string())))
            })
        },
        |_manager| {
            //
            Box::pin(async {})
        },
    )
    .await;

    register_cache_storages().await;
}

async fn register_cache_storages() {
    MemoryStore::register().await;
}
