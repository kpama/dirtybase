use dirtybase_contract::db::base::{
    connection::{ConnectionPoolRegisterTrait, ConnectionPoolTrait},
    manager::Manager,
    schema::{ClientType, DatabaseKind},
};
use dirtybase_contract::db::config::DirtybaseDbConfig;
use dirtybase_contract::db::event::SchemeWroteEvent;
use dirtybase_contract::db::event_handler::HandleSchemaWroteEvent;
use driver::{
    mysql::mysql_pool_manager::MySqlPoolManagerRegisterer,
    postgres::postgres_pool_manager::PostgresPoolManagerRegisterer,
    sqlite::sqlite_pool_manager::SqlitePoolManagerRegisterer,
};
use orsomafo::Dispatchable;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock},
};

pub(crate) static LAST_WRITE_TS: OnceLock<RwLock<HashMap<DatabaseKind, i64>>> = OnceLock::new();

pub mod driver;

pub use dirtybase_contract::db;
pub use dirtybase_contract::db::base;
pub use dirtybase_contract::db::dirtybase_db_types;
pub use dirtybase_contract::db::entity;
pub use dirtybase_contract::db::macros;
pub use dirtybase_contract::dirtybase_config;

pub type ConnectionsType = HashMap<DatabaseKind, HashMap<ClientType, Box<dyn ConnectionPoolTrait>>>;

#[derive(Debug, Clone)]
pub struct ConnectionPoolManager {
    pub(crate) connections: Arc<ConnectionsType>,
    pub(crate) config: DirtybaseDbConfig,
}

impl ConnectionPoolManager {
    pub async fn new(config: DirtybaseDbConfig) -> Self {
        let mut connections: ConnectionsType = HashMap::new();

        let mysql_pool_registerer = MySqlPoolManagerRegisterer;
        let sqlite_pool_registerer = SqlitePoolManagerRegisterer;
        let postgres_pool_registerer = PostgresPoolManagerRegisterer;

        // mysql
        if let Some(conn) = mysql_pool_registerer.register(&config).await {
            connections.insert(DatabaseKind::Mysql, conn);
        }

        // sqlite
        if let Some(conn) = sqlite_pool_registerer.register(&config).await {
            connections.insert(DatabaseKind::Sqlite, conn);
        }

        // postgres
        if let Some(conn) = postgres_pool_registerer.register(&config).await {
            connections.insert(DatabaseKind::Postgres, conn);
        }

        Self {
            connections: Arc::new(connections),
            config,
        }
    }

    pub fn default_schema_manager(&self) -> Result<Manager, anyhow::Error> {
        self.schema_manger(self.config.default.as_ref().unwrap())
    }

    pub fn default_kind(&self) -> &Option<DatabaseKind> {
        &self.config.default
    }

    pub fn schema_manger(&self, kind: &DatabaseKind) -> Result<Manager, anyhow::Error> {
        if self.connections.contains_key(kind) {
            Ok(Manager::new(
                self.connections.clone(),
                kind.clone(),
                self.config.clone(),
            ))
        } else {
            Err(anyhow::anyhow!(
                "Could not find connection pool: {:?}",
                kind
            ))
        }
    }
}

pub async fn setup(config: &dirtybase_config::DirtyConfig) -> ConnectionPoolManager {
    let base_config = DirtybaseDbConfig::new(config).await;

    LAST_WRITE_TS.get_or_init(|| RwLock::new(HashMap::new()));

    // event handlers
    _ = SchemeWroteEvent::subscribe::<HandleSchemaWroteEvent>().await;

    setup_using(base_config).await
}

pub async fn setup_using(config: DirtybaseDbConfig) -> ConnectionPoolManager {
    let pool_manager = ConnectionPoolManager::new(config).await;

    busybody::helpers::service_container().set_type(pool_manager.clone());

    pool_manager
}

#[busybody::async_trait]
impl busybody::Injectable for ConnectionPoolManager {
    async fn inject(container: &busybody::ServiceContainer) -> Self {
        container.get_type().unwrap()
    }
}
