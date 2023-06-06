use clap::{Parser, Subcommand};
use migration::MigrateDirection;

mod app;
mod http;
mod migration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();
    let args = Args::parse();

    match &args.command {
        Some(Commands::Serve) => {
            let dirty_app = app::setup().await;
            dirty_app.event_dispatcher().whisper("system_ready");
            let _ = http::init(dirty_app).await;
        }
        Some(Commands::Migrate { action }) => {
            let _dirty_app = app::setup().await;
            match action {
                MigrateDirection::Up => {
                    // TODO: implement migrating up
                    println!("migrating up");
                }
                MigrateDirection::Down => {
                    // TODO: implement migrating down
                    println!("migrating down");
                }
                MigrateDirection::New { name } => {
                    // TODO: implement creating a migration file
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
        action: MigrateDirection,
    },
}
