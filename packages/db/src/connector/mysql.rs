use std::{collections::HashMap, sync::Arc};

use dirtybase_contract::db::config::{BaseConfig, ConfigSet};
use mysql_pool_manager::MySqlPoolManagerRegisterer;

use crate::base::{
    connection::ConnectionPoolRegisterTrait, manager::Manager, schema::DatabaseKind,
};

pub mod mysql_pool_manager;
pub mod mysql_schema_manager;

/// Create a new manager using the configuration provided
pub async fn make_mysql_manager(base: BaseConfig) -> Manager {
    let mut config_set = ConfigSet::new();
    config_set.insert(base.client_type, base);
    let pools = MySqlPoolManagerRegisterer
        .register(&config_set)
        .await
        .unwrap();
    let mut connections = HashMap::new();
    connections.insert(DatabaseKind::Mysql, pools);
    Manager::new(
        Arc::new(connections),
        crate::base::schema::DatabaseKind::Mysql,
        config_set,
    )
}
