#![allow(dead_code)]

use clap::{Parser, Subcommand};

mod config;
mod dirtybase_app;
mod event_handler;
mod fields;

pub mod client;
pub mod core;
pub mod helper;
pub mod model;
pub mod pipeline;
pub mod setup_database;
pub mod setup_defaults;
pub mod token_claim;

pub use config::Config;
pub use dirtybase_app::DirtyBaseApp;

use crate::cli;
use crate::dirtybase_entry;
use crate::http;
use crate::migrator::MigrateAction;

use self::event_handler::register_event_handlers;

pub type DirtyBaseAppService = busybody::Service<dirtybase_app::DirtyBaseApp>;

/// Setup database application using configs in .env files
pub async fn setup() -> DirtyBaseAppService {
    let config = Config::default();
    // setup email adapters
    setup_using(&config).await
}

/// Setup database application using custom configuration
/// A builder exist that assist in building the configuration instance
/// ```rust
/// # use crate::dirtybase::app::ConfigBuilder;
/// let builder = ConfigBuilder::new();
/// let config = builder.app_name("My awesome application")
///                     .db_connection("...")
///                     .build();
/// ```
///
pub async fn setup_using(config: &Config) -> DirtyBaseAppService {
    let pool_manager = dirtybase_db::setup(config.dirty_config()).await;
    let cache_manager = dirtybase_cache::setup(config.dirty_config()).await;

    match dirtybase_app::DirtyBaseApp::new(config, pool_manager, cache_manager).await {
        Ok(app) => {
            register_event_handlers(orsomafo::EventDispatcherBuilder::new())
                .build()
                .await;

            // email adapters
            dirtybase_mail::register_mail_adapters().await;

            // app.db_setup().await;
            app.register(dirtybase_entry::Extension).await;

            app
        }
        Err(e) => {
            log::error!("server is not up: {}", e);
            panic!();
        }
    }
}

pub async fn run_http(app: DirtyBaseAppService) {
    _ = http::init(app).await;
}

pub async fn run_cli(app: DirtyBaseAppService, command: &Commands) {
    _ = cli::init(app, command).await;
}

pub async fn run(app: DirtyBaseAppService) {
    let args = Args::parse();

    match &args.command {
        Some(Commands::Serve) => run_http(app).await,
        Some(cmd) => run_cli(app, cmd).await,
        None => {
            println!("Unknown command")
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the application web server
    Serve,
    /// Execute migration
    Migrate {
        #[command(subcommand)]
        action: MigrateAction,
    },
    /// Process queued jobs
    Queue { name: Option<String> },
    /// Handle dispatched events
    Handle { cluster: Option<String> },
}
