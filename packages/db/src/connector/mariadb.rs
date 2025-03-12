pub mod mariadb_pool_manager;
pub mod mariadb_schema_manager;

use std::collections::HashMap;

use mariadb_pool_manager::MariadbPoolManagerRegisterer;
use mariadb_schema_manager::MARIADB_KIND;

const LOG_TARGET: &str = "mariadb_db_driver";

use crate::{
    ConnectionPoolRegisterTrait,
    base::{manager::Manager, schema::DatabaseKind},
    config::{ConfigSet, ConnectionConfig},
    make_manager,
};

/// Create a new manager using the configuration provided
pub async fn make_mysql_manager(base: ConnectionConfig) -> Manager {
    let mut config_set = ConfigSet::new();
    let kind: DatabaseKind = MARIADB_KIND.into();
    config_set.insert(base.client_type, base);
    let pools = MariadbPoolManagerRegisterer
        .register(&config_set)
        .await
        .unwrap();
    let mut connections = HashMap::new();
    connections.insert(kind.clone(), pools);

    make_manager(connections, kind, &config_set)
}
