use std::time::Duration;

use dirtybase_cache::{CacheManager, config::CacheConfig};
use dirtybase_contract::config::DirtyConfig;

#[tokio::main]
async fn main() {
    let dty_config = DirtyConfig::default();
    let config = CacheConfig::new(&dty_config).await;
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
