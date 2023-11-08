mod app;
mod cli;
mod dirtybase_entry;
mod http;
mod migrator;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let dirtybase_app = app::setup().await;
    //
    app::run(dirtybase_app).await;

    Ok(())
}
