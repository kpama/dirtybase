#![allow(unused)]
use std::collections::HashMap;

use dirtybase_config::DirtyConfig;

use crate::{
    cli::CliCommandManager,
    http::{MiddlewareManager, RouterManager},
};

pub type ExtensionMigrations = Vec<Box<dyn super::db::migration::Migration>>;

#[async_trait::async_trait]
pub trait ExtensionSetup: Send + Sync {
    /// Setup the extension
    async fn setup(&self, config: &DirtyConfig) {
        //..
    }

    async fn shutdown(&self) {
        log::debug!("shutting down extension: {}", self.id());
        // logic to run when the server is shutting down
    }

    /// Register HTTP routes
    fn register_routes(&self, mut manager: RouterManager) -> RouterManager {
        manager
    }

    /// Register Middlewares
    fn register_web_middlewares(&self, mut manager: MiddlewareManager) -> MiddlewareManager {
        manager
    }

    /// register cli sub commands
    fn register_cli_commands(&self, mut manager: CliCommandManager) -> CliCommandManager {
        manager
    }

    fn migrations(&self) -> ExtensionMigrations {
        Vec::new()
    }

    fn id(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
