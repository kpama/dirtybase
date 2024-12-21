use config::CacheConfig;

mod cache_dirtybase_entry;
mod cache_manager;

pub mod config;
pub mod model;

pub use cache_dirtybase_entry::*;
pub use cache_manager::cache_entry::CacheEntry;
pub use cache_manager::CacheManager;

pub async fn setup(config: &dirtybase_config::DirtyConfig) -> cache_manager::CacheManager {
    let cache_config = CacheConfig::new(config);
    setup_using(&cache_config).await
}

pub async fn setup_using(config: &config::CacheConfig) -> cache_manager::CacheManager {
    let manager = cache_manager::CacheManager::new(config).await;

    busybody::helpers::register_type(manager)
        .get_type()
        .unwrap()
}
