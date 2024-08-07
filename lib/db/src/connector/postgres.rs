use std::{collections::HashMap, sync::Arc};

use dirtybase_contract::db::config::{BaseConfig, ConfigSet};
use postgres_pool_manager::PostgresPoolManagerRegisterer;

use crate::base::{
    connection::ConnectionPoolRegisterTrait, manager::Manager, schema::DatabaseKind,
};

pub mod postgres_pool_manager;
pub mod postgres_schema_manager;

/// Create a new manager using the configuration provided
pub async fn make_postgres_manager(base: BaseConfig) -> Manager {
    let mut config_set = ConfigSet::new();
    config_set.insert(base.client_type, base);

    let pools = PostgresPoolManagerRegisterer
        .register(&config_set)
        .await
        .unwrap();

    let mut connections = HashMap::new();
    connections.insert(DatabaseKind::Postgres, pools);
    Manager::new(Arc::new(connections), DatabaseKind::Postgres, config_set)
}
