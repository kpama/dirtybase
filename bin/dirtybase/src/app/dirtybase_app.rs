use super::client::redis_client::make_redis_client;
use super::setup_defaults::setup_default_entities;
use super::Config;
use dirtybase_db::db::base::manager::Manager;
use dirtybase_db::db::entity::user::{UserRepository, UserService};
use dirtybase_db::ConnectionPoolManager;
use tokio::sync::RwLock;

pub struct DirtyBaseApp {
    config: Config,
    pool_manager: ConnectionPoolManager,
    cache_manager: dirtybase_cache::CacheManager,
    extensions: RwLock<Vec<Box<dyn dirtybase_contract::ExtensionSetup>>>,
}

impl DirtyBaseApp {
    pub async fn new(
        config: &Config,
        pool_manager: ConnectionPoolManager,
        cache_manager: dirtybase_cache::CacheManager,
    ) -> anyhow::Result<busybody::Service<Self>> {
        let instance = Self {
            pool_manager,
            cache_manager,
            config: config.clone(),
            extensions: RwLock::new(Vec::new()),
        };

        busybody::helpers::service_container().set(instance);
        make_redis_client(&config).await;

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

    pub async fn db_setup(&self) {
        setup_default_entities().await;
    }

    pub fn config(&self) -> Config {
        self.config.clone()
    }

    pub fn ref_config(&self) -> &Config {
        &self.config
    }
}
