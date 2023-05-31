use app::app_setup::DirtyBase;
use clap::{Parser, Subcommand};
use std::env;

pub mod app;
pub mod http;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    load_dot_env();
    pretty_env_logger::init();
    let args = Args::parse();

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

    let secret_key = if let Ok(key) = env::var("DTY_SECRET") {
        key
    } else {
        "".to_owned()
    };

    // app.db_setup().await;
    match &args.command {
        Some(Commands::Serve) => {
            match DirtyBase::new(&db_connection, max_connection, &secret_key).await {
                Ok(app) => {
                    app.db_setup().await;
                    let _ = http::init(app).await;
                }
                Err(e) => {
                    log::error!("server is not up: {}", e);
                    panic!();
                }
            }
        }
        Some(Commands::Migrate { action }) => {
            let _app = DirtyBase::new(&db_connection, max_connection, &secret_key)
                .await
                .unwrap();
            match action {
                Migrate::Up => {
                    println!("migrating up");
                }
                Migrate::Down => {
                    println!("migrating down");
                }
                Migrate::New { name } => {
                    println!("create a new migration: {}", name);
                }
            }
        }
        None => {
            println!("unknown sub command");
        }
    }
    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Serve,
    Migrate {
        #[command(subcommand)]
        action: Migrate,
    },
}

#[derive(Subcommand, Debug)]
enum Migrate {
    Up,
    Down,
    New { name: String },
}

fn load_dot_env() {
    let _ = dotenv::from_filename(".env.defaults");
    let _ = dotenv::from_filename(".env");
    let _ = dotenv::from_filename(".env.dev");
    let _ = dotenv::from_filename(".env.prod");
}
