mod extension;

pub mod app_contract;
pub mod auth_contract;
pub mod cli_contract;
pub mod config_contract;
pub mod db_contract;
pub mod http_contract;
pub mod multitenant_contract;
pub mod queue_contract;
pub mod session_contract;
pub mod user;

pub use async_trait::async_trait;
pub use axum;
pub use busybody;
pub use extension::ExtensionManager;
pub use extension::ExtensionMigrations;
pub use extension::ExtensionSetup;
pub use fama;
pub use serde;
pub use serde_json;

#[macro_export]
macro_rules! register_migration {
    () => {
        None
    };
    ($($m:expr),+ $(,)?) => {
        {
            let mut v = Vec::<Box<dyn ::dirtybase_contract::db_contract::migration::Migration>>::new();
            $(
                v.push(Box::new($m));
            )*
            Some(v)
        }
    };
}

pub mod prelude {
    pub use super::app_contract::*;
    pub use super::auth_contract::prelude::*;
    pub use super::cli_contract;
    pub use super::config_contract::*;
    pub use super::extension::ExtensionManager;
    pub use super::extension::ExtensionMigrations;
    pub use super::extension::ExtensionSetup;
    pub use super::http_contract::prelude::*;
    pub use chrono::*;

    pub use anyhow;
    pub use async_trait::async_trait;
    pub use busybody;
    pub use busybody::Resolver;
    pub use fama::PipelineBuilderTrait;
    pub use fama::PipelineTrait;
}
