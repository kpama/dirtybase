#![allow(unused)]
use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use axum::{extract::Request, response::Response};
use axum_extra::extract::{cookie, CookieJar};
use clap::ArgMatches;
use tokio::sync::RwLock;

use crate::{
    app::Context,
    cli::{CliCommandManager, CliMiddlewareManager},
    config::DirtyConfig,
    http::{RouterBuilder, RouterManager, WebMiddlewareManager},
};

pub(crate) static EXTENSION_COLLECTION: OnceLock<RwLock<Vec<Box<dyn ExtensionSetup>>>> =
    OnceLock::new();
pub(crate) static EXTENSIONS_READY: OnceLock<bool> = OnceLock::new();

pub type ExtensionMigrations = Vec<Box<dyn super::db::migration::Migration>>;

#[async_trait::async_trait]
pub trait ExtensionSetup: Send + Sync {
    /// Setup the extension
    ///
    /// First method that will be called.
    async fn setup(&mut self, global_context: &Context) {
        // --
    }

    /// boot
    ///
    /// Second method that will be called.
    async fn boot(&mut self, global_context: &Context) {
        // --
    }

    /// Run
    ///
    /// Third method that will be called.
    async fn run(&mut self, global_context: &Context) {
        // --
    }

    /// Shutdown when the application is shutting down
    async fn shutdown(&mut self, global_context: &Context) {
        log::debug!("shutting down extension: {}", self.id());
        // logic to run when the server is shutting down
    }

    /// Register HTTP routes
    fn register_routes(&self, mut manager: RouterManager) -> RouterManager {
        manager
    }

    fn register_routes2(&self, router: &mut RouterBuilder) {
        //-
    }

    /// Calls for each web requests
    ///
    /// Use this method to set things up for this particular request
    async fn on_web_request(&self, req: Request, context: Context, cookie: &CookieJar) -> Request {
        req
    }

    async fn on_web_response(
        &self,
        resp: Response,
        cookie_jar: CookieJar,
        context: Context,
    ) -> (Response, CookieJar) {
        (resp, cookie_jar)
    }

    /// Calls for a cli command
    async fn on_cli_command(&self, cmd: &str, matches: ArgMatches, context: Context) -> ArgMatches {
        matches
    }

    /// Register web Middlewares
    fn register_web_middlewares(&self, mut manager: WebMiddlewareManager) -> WebMiddlewareManager {
        manager
    }

    // Register cli middlewares
    fn register_cli_middlewares(&self, mut manager: CliMiddlewareManager) -> CliMiddlewareManager {
        manager
    }

    /// register cli sub commands
    fn register_cli_commands(&self, mut manager: CliCommandManager) -> CliCommandManager {
        manager
    }

    fn migrations(&self, context: &Context) -> Option<ExtensionMigrations> {
        None
    }

    fn id(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn the_id() -> &'static str
    where
        Self: Sized,
    {
        std::any::type_name::<Self>()
    }

    async fn register(self)
    where
        Self: Sized + 'static,
    {
        ExtensionManager::register(self).await;
    }

    fn global_container(&self) -> Arc<busybody::ServiceContainer> {
        busybody::helpers::service_container()
    }
}

#[derive(Debug, Clone)]
pub struct ExtensionManager;

impl ExtensionManager {
    pub async fn register<T>(extension: T)
    where
        T: ExtensionSetup + 'static,
    {
        Self::init();
        if let Some(list) = EXTENSION_COLLECTION.get() {
            let mut lock = list.write().await;
            lock.push(Box::new(extension));
        }
    }

    pub async fn is_ready() -> bool {
        EXTENSIONS_READY.get().is_some()
    }

    pub async fn setup_boot_run(context: &Context) {
        if EXTENSIONS_READY.get().is_some() {
            return;
        }

        Self::init();
        // setup
        if let Some(list) = EXTENSION_COLLECTION.get() {
            let mut w_lock = list.write().await;
            for ext in w_lock.iter_mut() {
                tracing::trace!("setup: {}", ext.id());
                ext.setup(context).await;
            }
        }

        // boot
        if let Some(list) = EXTENSION_COLLECTION.get() {
            let mut w_lock = list.write().await;
            for ext in w_lock.iter_mut() {
                ext.boot(context).await;
            }
        }

        // run
        if let Some(list) = EXTENSION_COLLECTION.get() {
            let mut w_lock = list.write().await;
            for ext in w_lock.iter_mut() {
                ext.run(context).await;
            }
        }

        _ = EXTENSIONS_READY.set(true);
    }

    pub async fn shutdown(context: &Context) {
        if let Some(list) = EXTENSION_COLLECTION.get() {
            let mut w_lock = list.write().await;
            for ext in w_lock.iter_mut() {
                ext.shutdown(context).await;
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

    pub async fn extensions_mut(mut callback: impl FnMut(&mut Box<dyn ExtensionSetup>)) {
        Self::init();
        if let Some(list) = EXTENSION_COLLECTION.get() {
            let mut lock = list.write().await;
            for ext in lock.iter_mut() {
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
