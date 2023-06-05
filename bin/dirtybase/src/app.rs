#![allow(dead_code)]

mod config;
mod entity;
mod fields;

pub mod dirtybase;
pub mod setup_database;
pub use config::Config;
pub use config::ConfigBuilder;
pub mod setup_defaults;

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
