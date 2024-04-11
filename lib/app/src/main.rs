#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let app_service = dirtybase_app::setup().await?;

    let result = dirtybase_app::run(app_service.clone()).await;

    result
}
