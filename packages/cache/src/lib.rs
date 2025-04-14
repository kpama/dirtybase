use config::CacheConfig;

mod cache_manager;
mod cache_storage_resolver;
mod dirtybase_entry;
mod resource_manager;

pub mod config;
pub mod model;
pub use cache_manager::cache_store;

pub use cache_manager::CacheManager;
pub use cache_manager::cache_entry::CacheEntry;
pub use cache_storage_resolver::*;
use dirtybase_contract::app_contract::Context;
pub use dirtybase_entry::*;
pub use resource_manager::*;

pub async fn setup(context: &Context) {
    let cache_config = context
        .get_config::<CacheConfig>("cache")
        .await
        .expect("could not configure cache manager");
    context.set(cache_config).await;

    register_resource_manager().await;
}
