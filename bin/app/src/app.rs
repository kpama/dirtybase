mod config;

pub(crate) mod setup_database;
pub(crate) mod setup_defaults;

pub mod command;
pub mod helper;
pub mod model;
pub mod pipeline;
pub mod token_claim;

pub use config::Config;
pub use config::ConfigBuilder;

use dirtybase_cache::CacheManager;
use dirtybase_contract::db::entity::user::{UserRepository, UserService};
use dirtybase_db::{base::manager::Manager, ConnectionPoolManager};
use tokio::sync::RwLock;

pub type AppService = busybody::Service<App>;

pub struct App {
    config: Config,
    pool_manager: ConnectionPoolManager,
    cache_manager: CacheManager,
    pub(crate) extensions: RwLock<Vec<Box<dyn dirtybase_contract::ExtensionSetup>>>,
}

impl App {
    pub async fn new(
        config: &Config,
        pool_manager: ConnectionPoolManager,
        cache_manager: dirtybase_cache::CacheManager,
    ) -> anyhow::Result<AppService> {
        let instance = Self {
            pool_manager,
            cache_manager,
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
        let lock = self.extensions.read().await;

        for ext in lock.iter() {
            ext.setup(self.config().dirty_config()).await;
        }
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
        self.pool_manager.default_schema_manager().unwrap()
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

    pub fn ref_config(&self) -> &Config {
        &self.config
    }
}
