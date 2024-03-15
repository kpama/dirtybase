#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let app_service = app::setup().await?;

    let result = app::run(app_service.clone()).await;

    result
}
