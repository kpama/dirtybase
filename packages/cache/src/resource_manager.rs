use dirtybase_contract::prelude::ContextResourceManager;

use crate::{CacheManager, config::CacheConfig};

pub async fn register_resource_manager() {
    ContextResourceManager::<CacheManager>::register(
        |context| {
            //
            Box::pin(async move {
                //...
                let name = context
                    .tenant()
                    .await
                    .expect("could not get tenant")
                    .id()
                    .to_string();

                (name, 0)
            })
        },
        |context| {
            //
            Box::pin(async move {
                let config = context.get_config::<CacheConfig>("cache").await.unwrap();

                Ok(CacheManager::new(&config).await)
            })
        },
        |_manager| {
            //
            Box::pin(async {})
        },
    )
    .await;
}
