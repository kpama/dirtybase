use std::{collections::HashMap, sync::Arc};

use sqlite_pool_manager::SqlitePoolManagerRegisterer;

use crate::{
    base::{connection::ConnectionPoolRegisterTrait, manager::Manager, schema::DatabaseKind},
    config::DirtybaseDbConfig,
};

pub mod sqlite_pool_manager;
pub mod sqlite_schema_manager;

/// Create a new manager using the configuration provided
pub async fn make_sqlite_manager(config: DirtybaseDbConfig) -> Manager {
    let pools = SqlitePoolManagerRegisterer.register(&config).await.unwrap();
    let mut connections = HashMap::new();
    connections.insert(DatabaseKind::Sqlite, pools);
    Manager::new(Arc::new(connections), DatabaseKind::Sqlite, config)
}

pub async fn make_sqlite_in_memory_manager() -> Manager {
    let config = DirtybaseDbConfig::in_memory();
    let pools = SqlitePoolManagerRegisterer.register(&config).await.unwrap();
    let mut connections = HashMap::new();
    connections.insert(DatabaseKind::Sqlite, pools);
    Manager::new(Arc::new(connections), DatabaseKind::Sqlite, config)
}
