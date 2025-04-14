use dirtybase_cache::CacheManager;
use dirtybase_contract::app_contract::Context;

#[tokio::main]
async fn main() {
    let ctx = Context::make_global().await;
    dirtybase_cache::setup(&ctx).await;
    let manager = ctx
        .get::<CacheManager>()
        .await
        .expect("could not get cache manager");

    manager
        .add(
            "message",
            "The quick brown fox jumps over the lazy dog",
            None,
        )
        .await;

    println!("result: {:?}", manager.get::<String>("message").await);
}
