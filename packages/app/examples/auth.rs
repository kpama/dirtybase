use dirtybase_app::setup;
use dirtybase_contract::{auth::StorageResolverPipeline, prelude::*};
use dirtybase_db::{base::manager::Manager, types::ArcUuid7};
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");

    let app_service = setup().await.unwrap();
    app_service.init().await;

    let global_context = app_service.global_context().await;
    let storage = StorageResolverPipeline::new(global_context.clone())
        .get_provider()
        .await
        .unwrap();

    let mut payload = AuthUserPayload::default();
    payload.username = Some("admin".to_string());
    payload.email = Some("example@yahoo.com".to_string());
    payload.rotate_salt = true;
    let x = storage.store(payload).await;

    println!(">>>>>>>>>> completed <<<<<<<<<<<<<");
}
