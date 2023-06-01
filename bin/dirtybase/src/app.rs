#![allow(dead_code)]

mod config;
mod entity;
mod fields;

pub mod dirtybase;
pub mod setup_database;
pub use config::Config;
pub use config::ConfigBuilder;

/// Loads configuration from .env files.
/// Multiple .env files are check in the following order
///  - .env.defaults
///  - .env
///  - .env.dev
///  - .env.prod
/// Values are merged from these files
fn load_dot_env() {
    let _ = dotenv::from_filename(".env.defaults");
    let _ = dotenv::from_filename(".env");
    let _ = dotenv::from_filename(".env.dev");
    let _ = dotenv::from_filename(".env.prod");
}

/// Setup database application using configs in .env files
pub async fn setup() -> dirtybase::DirtyBase {
    let config = Config::default();
    setup_using(config).await
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
pub async fn setup_using(config: Config) -> dirtybase::DirtyBase {
    load_dot_env();
    match dirtybase::DirtyBase::new(config).await {
        Ok(app) => {
            app.db_setup().await;
            app
        }
        Err(e) => {
            log::error!("server is not up: {}", e);
            panic!();
        }
    }
}
