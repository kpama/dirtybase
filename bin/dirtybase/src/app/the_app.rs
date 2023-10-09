use super::client::redis_client::make_redis_client;
use super::setup_database::create_data_tables;
use super::setup_defaults::setup_default_entities;
use super::Config;
use dirtybase_db::base::manager::Manager;
use dirtybase_db::entity::user::{UserRepository, UserService};
use dirtybase_db::ConnectionPoolManager;

pub struct DirtyBase {
    config: Config,
    pool_manager: ConnectionPoolManager,
    cache_manager: dirtybase_cache::CacheManager,
}

impl DirtyBase {
    pub async fn new(
        config: Config,
        pool_manager: ConnectionPoolManager,
        cache_manager: dirtybase_cache::CacheManager,
    ) -> anyhow::Result<busybody::Service<Self>> {
        let instance = Self {
            pool_manager,
            cache_manager,
            config: config.clone(),
        };

        busybody::helpers::service_container().set(instance);
        make_redis_client(&config).await;

        Ok(busybody::helpers::service_container()
            .get::<Self>()
            .unwrap())
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
        create_data_tables(self.schema_manger()).await;
        setup_default_entities().await;
    }

    pub fn config(&self) -> Config {
        self.config.clone()
    }

    pub fn ref_config(&self) -> &Config {
        &self.config
    }
}
