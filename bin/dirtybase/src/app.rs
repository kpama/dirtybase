#![allow(dead_code)]

mod config;
mod event_handler;
mod fields;
mod the_app;

pub mod client;
pub mod core;
pub mod helper;
pub mod model;
pub mod pipeline;
pub mod setup_database;
pub mod setup_defaults;
pub mod token_claim;

pub use config::Config;
pub use config::ConfigBuilder;
pub use the_app::DirtyBase;

use self::event_handler::register_event_handlers;

/// Setup database application using configs in .env files
pub async fn setup() -> busybody::Service<the_app::DirtyBase> {
    let config = Config::default();
    // setup email adapters
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
pub async fn setup_using(config: Config) -> busybody::Service<the_app::DirtyBase> {
    let pool_manager = dirtybase_db::setup(config.dirty_config()).await;
    let cache_manager = dirtybase_cache::setup(config.dirty_config()).await;

    match the_app::DirtyBase::new(config, pool_manager, cache_manager).await {
        Ok(app) => {
            register_event_handlers(orsomafo::EventDispatcherBuilder::new())
                .build()
                .await;

            // email adapters
            dirtybase_mail::register_mail_adapters().await;

            app.db_setup().await;
            app
        }
        Err(e) => {
            log::error!("server is not up: {}", e);
            panic!();
        }
    }
}
