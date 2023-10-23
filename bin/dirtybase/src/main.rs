mod app;
mod cli;
mod http;
mod migration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let dirtybase_app = app::setup().await;
    //
    app::run(dirtybase_app).await;

    Ok(())
}
