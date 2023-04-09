pub mod app;
pub mod http;

use app::app_setup::DirtyBase;
use clap::{Parser, Subcommand};
use dirtybase_db::driver::surreal::SurrealDbConfig;
use dotenv::dotenv;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();
    let args = Args::parse();
    let mut env_file_exist = true;

    if let Err(e) = dotenv() {
        log::error!("could not load .env file: {:#}", e);
        env_file_exist = false;
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

    let surreal_config = SurrealDbConfig::new_from_env();

    // let db = dirtybase_db::driver::surreal::setup("surrealdb:8000", "root", "root", "test", "test")
    //     .await;
    // let result = db
    //     .query("select * from family")
    //     .await
    //     .expect("could not query db");
    // dbg!(result);
    // return Ok(());

    //  app.db_setup().await;
    match &args.command {
        Some(Commands::Serve) => {
            if env_file_exist {
                match DirtyBase::new(&db_connection, max_connection, surreal_config).await {
                    Ok(app) => {
                        println!("serve the application");
                        let _ = http::init(app).await;
                    }
                    Err(e) => {
                        log::error!("server is not up: {}", e);
                    }
                }
            } else {
                println!("serve setup interface");
            }
        }
        Some(Commands::Migrate { action }) => {
            let _app = DirtyBase::new(&db_connection, max_connection, surreal_config)
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
