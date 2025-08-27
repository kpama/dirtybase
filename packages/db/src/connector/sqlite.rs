use std::collections::HashMap;

use sqlite_connector::SQLITE_KIND;
use sqlite_pool_manager::resolve;

use crate::{
    base::{manager::Manager, schema::DatabaseKind},
    config::{ConfigSet, ConnectionConfig},
    make_manager,
};

pub mod sqlite_connector;
pub mod sqlite_pool_manager;

const LOG_TARGET: &str = "sqlite_db_driver";

/// Create a new manager using the configuration provided
pub async fn make_sqlite_manager(base: ConnectionConfig) -> Manager {
    let mut config_set = ConfigSet::new();

    config_set.insert(base.client_type, base);
    make_sqlite_manager_from_set(config_set).await
}

pub async fn make_sqlite_manager_from_set(config_set: ConfigSet) -> Manager {
    let kind: DatabaseKind = SQLITE_KIND.into();
    let pools = resolve(&config_set).await.unwrap();
    let mut connections = HashMap::new();
    connections.insert(kind.clone(), pools);

    make_manager(connections, kind, &config_set)
}

pub async fn make_sqlite_in_memory_manager() -> Manager {
    let config_set = ConnectionConfig::in_memory_set();
    let kind: DatabaseKind = SQLITE_KIND.into();

    let pools = resolve(&config_set).await.unwrap();
    let mut connections = HashMap::new();
    connections.insert(kind.clone(), pools);

    make_manager(connections, kind, &config_set)
}
