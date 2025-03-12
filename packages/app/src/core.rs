mod config;

pub mod command;
pub mod model;

use std::convert::Infallible;
use std::ops::Deref;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

pub use config::Config;
pub use config::ConfigBuilder;

use dirtybase_contract::ExtensionManager;
use dirtybase_contract::app::Context;
use dirtybase_contract::config::DirtyConfig;

pub type AppService = busybody::Service<App>;

pub struct App {
    config: Config,
}

impl App {
    pub async fn new(config: &Config) -> anyhow::Result<AppService> {
        let instance = Self {
            config: config.clone(),
        };

        busybody::helpers::service_container().set(instance).await;

        Ok(busybody::helpers::service_container()
            .get::<Self>()
            .await
            .unwrap())
    }

    pub async fn register(
        &self,
        extension: impl dirtybase_contract::ExtensionSetup + 'static,
    ) -> &Self {
        ExtensionManager::register(extension).await;
        self
    }

    pub async fn global_context(&self) -> Context {
        dirtybase_contract::app::global_context().await
    }
    pub async fn init(&self) {
        ExtensionManager::setup_boot_run(&self.global_context().await).await;
    }

    pub async fn shutdown(&self) {
        log::info!("Shutting down");
        ExtensionManager::shutdown(&self.global_context().await).await;
    }

    pub async fn extensions(
        &self,
        callback: impl FnMut(&Box<dyn dirtybase_contract::ExtensionSetup>),
    ) {
        ExtensionManager::extensions(callback).await;
    }

    pub fn config(&self) -> Config {
        self.config.clone()
    }

    pub fn dirty_config(&self) -> DirtyConfig {
        self.config.dirty_config().clone()
    }

    pub fn dirty_config_ref(&self) -> &DirtyConfig {
        self.config.dirty_config()
    }

    pub fn config_ref(&self) -> &Config {
        &self.config
    }
}

pub struct AppServiceExtractor(AppService);

impl AppServiceExtractor {
    pub fn inner(self) -> AppService {
        self.0
    }
}

impl Deref for AppServiceExtractor {
    type Target = AppService;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<AppServiceExtractor> for AppService {
    fn from(value: AppServiceExtractor) -> Self {
        value.0
    }
}

impl<S> FromRequestParts<S> for AppServiceExtractor
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(AppServiceExtractor(
            busybody::helpers::get_service().await.unwrap(),
        ))
    }
}
