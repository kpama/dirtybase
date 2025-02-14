mod config;

pub(crate) mod setup_database;
pub(crate) mod setup_defaults;

pub mod command;
pub mod helper;
pub mod model;
pub mod pipeline;
pub mod token_claim;

use std::convert::Infallible;
use std::ops::Deref;
use std::sync::OnceLock;

use anyhow::anyhow;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

pub use config::Config;
pub use config::ConfigBuilder;

use dirtybase_contract::config::DirtyConfig;
use dirtybase_contract::ExtensionManager;
use dirtybase_db::base::manager::Manager;
use dirtybase_db::config::BaseConfig;
use dirtybase_db::connection_bus::MakePoolManagerCommand;
use dirtybase_user::entity::user::UserRepository;
use dirtybase_user::entity::user::UserService;

pub type AppService = busybody::Service<App>;

pub struct App {
    config: Config,
    default_db_manager: OnceLock<Result<Manager, anyhow::Error>>,
    is_ready: OnceLock<bool>,
}

impl App {
    pub async fn new(config: &Config) -> anyhow::Result<AppService> {
        let instance = Self {
            default_db_manager: OnceLock::new(),
            is_ready: OnceLock::new(),
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

    pub async fn init(&self) {
        if self.is_ready.get().is_some() {
            return;
        }

        ExtensionManager::setup_boot_run(self.config().dirty_config()).await;

        _ = self.is_ready.set(true)
    }

    pub async fn shutdown(&self) {
        log::info!("Shutting down");
        ExtensionManager::shutdown().await;
    }

    pub async fn extensions(
        &self,
        callback: impl FnMut(&Box<dyn dirtybase_contract::ExtensionSetup>),
    ) {
        ExtensionManager::extensions(callback).await;
    }

    pub async fn schema_manger(&self) -> Manager {
        self.try_schema_manager().await.unwrap()
    }

    pub async fn try_schema_manager(&self) -> Result<Manager, anyhow::Error> {
        let config = &self.config;
        let config_set = BaseConfig::set_from(config.dirty_config()).await;
        match self
            .default_db_manager
            .get_or_init(|| MakePoolManagerCommand::make_sync(config_set))
        {
            Ok(manager) => Ok(manager.clone()),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub async fn user_service(&self) -> UserService {
        UserService::new(UserRepository::new(self.schema_manger().await))
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
        Ok(AppServiceExtractor(busybody::helpers::provide().await))
    }
}
