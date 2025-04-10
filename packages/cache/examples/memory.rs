use std::time::Duration;

use dirtybase_cache::{CacheManager, config::CacheConfig};
use dirtybase_contract::{
    app_contract::Context,
    config_contract::{DirtyConfig, TryFromDirtyConfig},
};

#[tokio::main]
async fn main() {
    let dty_config = DirtyConfig::default();
    let ctx = Context::make_global().await;
    let config = CacheConfig::from_config(&dty_config, &ctx).await.unwrap();
    let manager = CacheManager::new(&config).await;

    manager
        .add(
            "message",
            "The quick brown fox jumps over the lazy dog",
            None,
        )
        .await;

    dbg!("result: {:?}", manager.get::<String>("message").await);
    let ts = Duration::from_secs(5);
    println!("{}", ts.as_secs());
}
