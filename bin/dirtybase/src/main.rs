pub mod app;
pub mod http;

use app::app_setup::Dirtybase;
use clap::Parser;
use dotenv::dotenv;
use pretty_env_logger;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();
    let args = Args::parse();

    if let Err(e) = dotenv() {
        log::error!("could not load .env file: {:#}", e);
    }

    let db_connection = if let Ok(conn) = env::var("DTY_DATABASE") {
        conn
    } else {
        log::error!("Error getting database connection string");
        "".to_owned()
    };

    let max_connection: u32 = if let Ok(max) = env::var("DTY_DATABASE_MAX_POOL_CONNECTION") {
        max.parse().unwrap_or(5)
    } else {
        5
    };

    let app = Dirtybase::new(&db_connection, max_connection)
        .await
        .unwrap();

    app.db_setup().await;

    if args.serve {
        http::init(app).await
    } else {
        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    serve: bool,
}
