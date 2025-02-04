mod extension;

pub mod app;
pub mod auth;
pub mod cli;
pub mod config;
pub mod db;
pub mod http;
pub mod queue;
pub mod session;
pub mod user;

pub use async_trait::async_trait;
pub use axum;
pub use extension::ExtensionManager;
pub use extension::ExtensionMigrations;
pub use extension::ExtensionSetup;
pub use serde;
pub use serde_json;

pub mod prelude {
    pub use super::auth::prelude::*;
    pub use super::cli::prelude::*;
    pub use super::extension::ExtensionManager;
    pub use super::extension::ExtensionMigrations;
    pub use super::extension::ExtensionSetup;
    pub use super::http::prelude::*;

    pub use async_trait::async_trait;
}
