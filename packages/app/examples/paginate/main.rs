use dirtybase_app::{run, setup};
use dirtybase_contract::ExtensionSetup;
use tracing_subscriber::EnvFilter;

use crate::product::ProductApp;

mod product;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        // .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");

    let app_service = setup().await.unwrap();

    ProductApp.register().await;

    _ = run(app_service).await;
}
