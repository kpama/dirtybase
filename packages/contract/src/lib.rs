mod extension;

pub mod app;
pub mod auth;
pub mod cli;
pub mod config;
pub mod db;
pub mod http;
pub mod multitenant;
pub mod queue;
pub mod session;
pub mod user;

pub use async_trait::async_trait;
pub use axum;
pub use busybody;
pub use extension::ExtensionManager;
pub use extension::ExtensionMigrations;
pub use extension::ExtensionSetup;
pub use serde;
pub use serde_json;

#[macro_export]
macro_rules! register_migration {
    () => {
        None
    };
    ($($m:expr),+ $(,)?) => {
        {
            let mut v = Vec::<::dirtybase_contract::db::migration::Migration>::new();
            $(
                v.push(Box::new($m));
            )*
            Some(v)
        }
    };
}

pub mod prelude {
    pub use super::auth::prelude::*;
    pub use super::cli::prelude::*;
    pub use super::config::*;
    pub use super::extension::ExtensionManager;
    pub use super::extension::ExtensionMigrations;
    pub use super::extension::ExtensionSetup;
    pub use super::http::prelude::*;

    pub use async_trait::async_trait;
}
