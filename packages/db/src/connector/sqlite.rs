use std::collections::HashMap;

use sqlite_pool_manager::SqlitePoolManagerRegisterer;
use sqlite_schema_manager::SQLITE_KIND;

use crate::{
    base::{manager::Manager, schema::DatabaseKind},
    config::{BaseConfig, ConfigSet},
    make_manager, ConnectionPoolRegisterTrait,
};

pub mod sqlite_pool_manager;
pub mod sqlite_schema_manager;

/// Create a new manager using the configuration provided
pub async fn make_sqlite_manager(base: BaseConfig) -> Manager {
    let mut config_set = ConfigSet::new();
    let kind: DatabaseKind = SQLITE_KIND.into();

    config_set.insert(base.client_type, base);

    let pools = SqlitePoolManagerRegisterer
        .register(&config_set)
        .await
        .unwrap();
    let mut connections = HashMap::new();
    connections.insert(kind.clone(), pools);

    make_manager(connections, kind, &config_set)
}

pub async fn make_sqlite_in_memory_manager() -> Manager {
    let config_set = BaseConfig::in_memory_set();
    let kind: DatabaseKind = SQLITE_KIND.into();

    let pools = SqlitePoolManagerRegisterer
        .register(&config_set)
        .await
        .unwrap();
    let mut connections = HashMap::new();
    connections.insert(kind.clone(), pools);

    make_manager(connections, kind, &config_set)
}
