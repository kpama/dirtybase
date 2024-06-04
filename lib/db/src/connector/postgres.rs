use std::{collections::HashMap, sync::Arc};

use postgres_pool_manager::PostgresPoolManagerRegisterer;

use crate::{
    base::{connection::ConnectionPoolRegisterTrait, manager::Manager, schema::DatabaseKind},
    config::DirtybaseDbConfig,
};

pub mod postgres_pool_manager;
pub mod postgres_schema_manager;

/// Create a new manager using the configuration provided
pub async fn make_postgres_manager(config: DirtybaseDbConfig) -> Manager {
    let pools = PostgresPoolManagerRegisterer
        .register(&config)
        .await
        .unwrap();
    let mut connections = HashMap::new();
    connections.insert(DatabaseKind::Postgres, pools);
    Manager::new(Arc::new(connections), DatabaseKind::Postgres, config)
}
