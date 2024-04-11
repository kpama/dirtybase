use std::{collections::HashMap, sync::Arc};

use crate::{
    base::{
        connection::{ConnectionPoolRegisterTrait, ConnectionPoolTrait},
        manager::Manager,
        schema::{ClientType, DatabaseKind},
    },
    config::DirtybaseDbConfig,
    connector::{
        mysql::mysql_pool_manager::MySqlPoolManagerRegisterer,
        postgres::postgres_pool_manager::PostgresPoolManagerRegisterer,
        sqlite::sqlite_pool_manager::SqlitePoolManagerRegisterer,
    },
};

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
