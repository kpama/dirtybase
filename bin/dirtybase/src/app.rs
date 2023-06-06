#![allow(dead_code)]

mod config;
mod entity;
mod event;
mod event_subscribers;
mod fields;
mod the_app;

pub mod setup_database;
pub mod setup_defaults;

pub use config::Config;
pub use config::ConfigBuilder;
pub use event::*;
pub use the_app::{DirtyBase, DirtyBaseWeb};

/// Setup database application using configs in .env files
pub async fn setup() -> the_app::DirtyBase {
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
pub async fn setup_using(config: Config) -> the_app::DirtyBase {
    match the_app::DirtyBase::new(config).await {
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
