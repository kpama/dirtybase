use std::collections::HashMap;

use mysql_pool_manager::resolve;
use mysql_schema_manager::MYSQL_KIND;

use crate::{
    base::{manager::Manager, schema::DatabaseKind},
    config::{ConfigSet, ConnectionConfig},
    make_manager,
};

pub mod mysql_pool_manager;
pub mod mysql_schema_manager;

/// Create a new manager using the configuration provided
pub async fn make_mysql_manager(base: ConnectionConfig) -> Manager {
    let mut config_set = ConfigSet::new();
    let kind: DatabaseKind = MYSQL_KIND.into();
    config_set.insert(base.client_type, base);
    let pools = resolve(&config_set).await.unwrap();
    let mut connections = HashMap::new();
    connections.insert(kind.clone(), pools);

    make_manager(connections, kind, &config_set)
}
