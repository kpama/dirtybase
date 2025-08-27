use std::collections::HashMap;

use postgres_connector::POSTGRES_KIND;
use postgres_pool_manager::resolve;

use crate::{
    base::{manager::Manager, schema::DatabaseKind},
    config::{ConfigSet, ConnectionConfig},
    make_manager,
};

pub mod postgres_connector;
pub mod postgres_pool_manager;

const LOG_TARGET: &str = "postgres_db_driver";

/// Create a new manager using the configuration provided
pub async fn make_postgres_manager(base: ConnectionConfig) -> Manager {
    let mut config_set = ConfigSet::new();
    let kind: DatabaseKind = POSTGRES_KIND.into();
    config_set.insert(base.client_type, base);

    let pools = resolve(&config_set)
        .await
        .expect("could not create postgres pool");

    let mut connections = HashMap::new();
    connections.insert(kind.clone(), pools);

    make_manager(connections, kind, &config_set)
}
