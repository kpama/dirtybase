mod config;
mod context;

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
pub use context::*;

pub use config::Config;
pub use config::ConfigBuilder;

use dirtybase_cache::CacheManager;
// use dirtybase_contract::db::entity::user::{UserRepository, UserService};
use dirtybase_db::base::manager::Manager;
use dirtybase_db::connection_bus::MakePoolManagerCommand;
use dirtybase_user::entity::user::UserRepository;
use dirtybase_user::entity::user::UserService;
use tokio::sync::RwLock;

pub type AppService = busybody::Service<App>;

pub struct App {
    config: Config,
    default_db_manager: OnceLock<Result<Manager, anyhow::Error>>,
    cache_manager: CacheManager,
    is_ready: OnceLock<bool>,
    pub(crate) extensions: RwLock<Vec<Box<dyn dirtybase_contract::ExtensionSetup>>>,
}

impl App {
    pub async fn new(
        config: &Config,
        cache_manager: dirtybase_cache::CacheManager,
    ) -> anyhow::Result<AppService> {
        let instance = Self {
            cache_manager,
            default_db_manager: OnceLock::new(),
            is_ready: OnceLock::new(),
            config: config.clone(),
            extensions: RwLock::new(Vec::new()),
        };

        busybody::helpers::service_container().set(instance);

        Ok(busybody::helpers::service_container()
            .get::<Self>()
            .unwrap())
    }

    pub async fn register(
        &self,
        extension: impl dirtybase_contract::ExtensionSetup + 'static,
    ) -> &Self {
        let mut lock = self.extensions.write().await;

        lock.push(Box::new(extension));

        self
    }

    pub async fn init(&self) {
        if self.is_ready.get().is_some() {
            return;
        }

        let lock = self.extensions.read().await;

        for ext in lock.iter() {
            ext.setup(self.config().dirty_config()).await;
        }

        _ = self.is_ready.set(true)
    }

    pub async fn shutdown(&self) {
        log::info!("Shutting down");

        // TODO: shutdown core resources

        let lock = self.extensions.read().await;
        for ext in lock.iter() {
            ext.shutdown().await;
        }
    }

    pub async fn extensions(
        &self,
        mut callback: impl FnMut(&Box<dyn dirtybase_contract::ExtensionSetup>),
    ) {
        let lock = self.extensions.read().await;
        for ext in lock.iter() {
            callback(ext);
        }
    }

    pub fn schema_manger(&self) -> Manager {
        self.try_schema_manager().unwrap()
    }

    pub fn try_schema_manager(&self) -> Result<Manager, anyhow::Error> {
        let config = &self.config;
        match self.default_db_manager.get_or_init(|| {
            let dirty_config = config.dirty_config().clone();
            MakePoolManagerCommand::make_sync(dirtybase_contract::db::config::BaseConfig::set_from(
                &dirty_config,
            ))
        }) {
            Ok(manager) => Ok(manager.clone()),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub fn cache_manager(&self) -> &dirtybase_cache::CacheManager {
        &self.cache_manager
    }

    pub fn user_service(&self) -> UserService {
        UserService::new(UserRepository::new(self.schema_manger()))
    }

    pub fn config(&self) -> Config {
        self.config.clone()
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

impl Into<AppService> for AppServiceExtractor {
    fn into(self) -> AppService {
        self.0
    }
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AppServiceExtractor
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(AppServiceExtractor(busybody::helpers::provide().await))
    }
}
