use config::CacheConfig;

mod cache_dirtybase_entry;
mod cache_manager;

pub mod config;
pub mod model;

pub use cache_dirtybase_entry::*;
pub use cache_manager::cache_entry::CacheEntry;
pub use cache_manager::CacheManager;
use dirtybase_contract::config::DirtyConfig;

pub async fn setup(config: &DirtyConfig) -> cache_manager::CacheManager {
    let cache_config = CacheConfig::new(config).await;
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
