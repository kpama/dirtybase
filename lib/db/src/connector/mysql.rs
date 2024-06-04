use std::{collections::HashMap, sync::Arc};

use mysql_pool_manager::MySqlPoolManagerRegisterer;

use crate::{
    base::{connection::ConnectionPoolRegisterTrait, manager::Manager, schema::DatabaseKind},
    config::DirtybaseDbConfig,
};

pub mod mysql_pool_manager;
pub mod mysql_schema_manager;

/// Create a new manager using the configuration provided
pub async fn make_mysql_manager(config: DirtybaseDbConfig) -> Manager {
    let pools = MySqlPoolManagerRegisterer.register(&config).await.unwrap();
    let mut connections = HashMap::new();
    connections.insert(DatabaseKind::Mysql, pools);
    Manager::new(
        Arc::new(connections),
        crate::base::schema::DatabaseKind::Mysql,
        config,
    )
}
