use config::CacheConfig;

mod cache_manager;
mod cache_storage_resolver;
mod dirtybase_entry;
mod resource_manager;

pub mod config;
pub mod model;

pub use cache_manager::CacheManager;
pub use cache_manager::cache_entry::CacheEntry;
pub use cache_storage_resolver::*;
use dirtybase_contract::app_contract::Context;
pub use dirtybase_entry::*;
pub use resource_manager::*;

pub async fn setup(context: &Context) -> cache_manager::CacheManager {
    let cache_config = context
        .get_config::<CacheConfig>("dirtybase::cache")
        .await
        .unwrap();
    setup_using(&cache_config).await
}

pub async fn setup_using(config: &config::CacheConfig) -> cache_manager::CacheManager {
    let manager = cache_manager::CacheManager::new(config).await;

    busybody::helpers::register_type(manager)
        .await
        .get_type()
        .await
        .unwrap()
}
