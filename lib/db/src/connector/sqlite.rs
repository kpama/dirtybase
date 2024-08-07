use std::{collections::HashMap, sync::Arc};

use dirtybase_contract::db::config::{BaseConfig, ConfigSet};
use sqlite_pool_manager::SqlitePoolManagerRegisterer;

use crate::base::{
    connection::ConnectionPoolRegisterTrait, manager::Manager, schema::DatabaseKind,
};

pub mod sqlite_pool_manager;
pub mod sqlite_schema_manager;

/// Create a new manager using the configuration provided
pub async fn make_sqlite_manager(base: BaseConfig) -> Manager {
    let mut config_set = ConfigSet::new();
    config_set.insert(base.client_type, base);

    let pools = SqlitePoolManagerRegisterer
        .register(&config_set)
        .await
        .unwrap();
    let mut connections = HashMap::new();
    connections.insert(DatabaseKind::Sqlite, pools);
    Manager::new(Arc::new(connections), DatabaseKind::Sqlite, config_set)
}

pub async fn make_sqlite_in_memory_manager() -> Manager {
    let config_set = BaseConfig::in_memory_set();

    let pools = SqlitePoolManagerRegisterer
        .register(&config_set)
        .await
        .unwrap();
    let mut connections = HashMap::new();
    connections.insert(DatabaseKind::Sqlite, pools);
    Manager::new(Arc::new(connections), DatabaseKind::Sqlite, config_set)
}
