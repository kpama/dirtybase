use super::setup_database::create_data_tables;
use super::setup_defaults::setup_default_entities;
use super::Config;
use dirtybase_db::base::connection::ConnectionPoolRegisterTrait;
use dirtybase_db::base::manager::Manager;
use dirtybase_db::driver::mysql::mysql_pool_manager::MySqlPoolManagerRegisterer;
use dirtybase_db::driver::sqlite::sqlite_pool_manager::SqlitePoolManagerRegisterer;
use dirtybase_db::entity::user::{UserRepository, UserService};
use dirtybase_db::ConnectionPoolManager;
use sqlx::any::AnyKind;
use std::str::FromStr;

#[derive(Debug)]
pub struct DirtyBase {
    default_db: String,
    config: Config,
    pool_manager: ConnectionPoolManager,
}

impl DirtyBase {
    pub async fn new(config: Config) -> anyhow::Result<busybody::Service<Self>> {
        let mut connection_pools: Vec<Box<dyn ConnectionPoolRegisterTrait>> = Vec::new();
        let default;

        match AnyKind::from_str(config.db_connection()) {
            Ok(kind) => match kind {
                AnyKind::MySql => {
                    connection_pools.push(Box::new(MySqlPoolManagerRegisterer));
                    default = "mysql"
                }
                AnyKind::Sqlite => {
                    connection_pools.push(Box::new(SqlitePoolManagerRegisterer));
                    default = "sqlite"
                }
            },
            Err(_) => panic!("Could not determine database kind"),
        }

        let pool_manager = ConnectionPoolManager::new(
            connection_pools,
            default,
            &config.db_connection(),
            config.max_db_pool(),
        )
        .await;

        let instance = Self {
            default_db: default.into(),
            pool_manager,
            config,
        };

        busybody::helpers::service_container().set(instance);
        Ok(busybody::helpers::service_container()
            .get::<Self>()
            .unwrap())
    }

    pub fn default_db(&self) -> &String {
        &self.default_db
    }

    pub fn schema_manger(&self) -> Manager {
        self.pool_manager.default_schema_manager().unwrap()
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
}
