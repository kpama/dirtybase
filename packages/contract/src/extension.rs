#![allow(unused)]
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use axum::extract::Request;
use clap::ArgMatches;
use dirtybase_config::DirtyConfig;
use tokio::sync::RwLock;

use crate::{
    cli::CliCommandManager,
    http::{RouterManager, WebMiddlewareManager},
};

pub(crate) static EXTENSION_COLLECTION: OnceLock<RwLock<Vec<Box<dyn ExtensionSetup>>>> =
    OnceLock::new();

pub type ExtensionMigrations = Vec<Box<dyn super::db::migration::Migration>>;

#[async_trait::async_trait]
pub trait ExtensionSetup: Send + Sync {
    /// Setup the extension
    async fn setup(&self, config: &DirtyConfig) {
        // --
    }

    /// boot
    async fn boot(&self) {
        // --
    }

    /// Run
    async fn run(&self) {
        // --
    }

    async fn shutdown(&self) {
        log::debug!("shutting down extension: {}", self.id());
        // logic to run when the server is shutting down
    }

    /// Register HTTP routes
    fn register_routes(
        &self,
        mut manager: RouterManager,
        middleware_manager: &WebMiddlewareManager,
    ) -> RouterManager {
        manager
    }

    /// Calls for each web requests
    async fn on_web_request(&self, req: Request) -> Request {
        req
    }

    /// Calls for a cli command
    async fn on_cli_command(&self, cmd: &str, matches: ArgMatches) -> ArgMatches {
        matches
    }

    /// Register web Middlewares
    fn register_web_middlewares(&self, mut manager: WebMiddlewareManager) -> WebMiddlewareManager {
        manager
    }

    // Rgister cli middlewares

    /// register cli sub commands
    fn register_cli_commands(&self, mut manager: CliCommandManager) -> CliCommandManager {
        manager
    }

    // TODO: Make the returned type an option
    fn migrations(&self) -> ExtensionMigrations {
        Vec::new()
    }

    fn id(&self) -> &str {
        std::any::type_name::<Self>()
    }

    async fn register(self)
    where
        Self: Sized + 'static,
    {
        ExtensionManager::register(self).await;
    }
}

#[derive(Debug, Clone)]
pub struct ExtensionManager;

impl ExtensionManager {
    pub async fn register(extension: impl ExtensionSetup + 'static) {
        Self::init();
        if let Some(list) = EXTENSION_COLLECTION.get() {
            let mut lock = list.write().await;
            lock.push(Box::new(extension));
        }
    }

    pub async fn setup_boot_run(config: &DirtyConfig) {
        Self::init();
        // setup
        if let Some(list) = EXTENSION_COLLECTION.get() {
            let lock = list.read().await;
            for ext in lock.iter() {
                ext.setup(config).await;
            }
        }

        // boot
        if let Some(list) = EXTENSION_COLLECTION.get() {
            let lock = list.read().await;
            for ext in lock.iter() {
                ext.boot().await;
            }
        }

        // run
        if let Some(list) = EXTENSION_COLLECTION.get() {
            let lock = list.read().await;
            for ext in lock.iter() {
                ext.run().await;
            }
        }
    }

    pub async fn shutdown() {
        if let Some(list) = EXTENSION_COLLECTION.get() {
            let lock = list.read().await;
            for ext in lock.iter() {
                ext.shutdown().await;
            }
        }
    }

    pub async fn extensions(mut callback: impl FnMut(&Box<dyn ExtensionSetup>)) {
        Self::init();

        if let Some(list) = EXTENSION_COLLECTION.get() {
            let lock = list.read().await;
            for ext in lock.iter() {
                callback(ext);
            }
        }
    }

    pub fn list() -> &'static RwLock<Vec<Box<dyn ExtensionSetup>>> {
        Self::init();
        EXTENSION_COLLECTION.get().unwrap()
    }

    pub(crate) fn init() {
        let manager = EXTENSION_COLLECTION.get_or_init(Default::default);
    }
}
