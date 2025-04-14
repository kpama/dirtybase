pub mod config;
pub mod connector;
pub mod pool_manager_resolver;

mod command;
mod dirtybase_entry;
mod model;
mod resource_manager;

use std::sync::Arc;

use base::schema::DatabaseKind;
use config::ConfigSet;
use connector::{
    mariadb::mariadb_pool_manager::MariadbPoolManager, mysql::mysql_pool_manager::MysqlPoolManager,
    postgres::postgres_pool_manager::PostgresPoolManager,
    sqlite::sqlite_pool_manager::SqlitePoolManager,
};
use dirtybase_contract::db_contract::base::{manager::Manager, schema::ClientType};
pub use dirtybase_contract::db_contract::*;
pub use dirtybase_entry::*;

pub use anyhow;

pub const USER_TABLE: &str = "core_user";

pub fn make_manager(
    connections: DatabaseKindPoolCollection,
    kind: DatabaseKind,
    config_set: &ConfigSet,
) -> Manager {
    let mut is_writable = false;
    let mut write_is_sticky = false;
    let mut sticky_duration = 0;

    if let Some(config) = config_set.get(&ClientType::Write) {
        write_is_sticky = config.sticky.unwrap_or_default();
        sticky_duration = config.sticky_duration.unwrap_or_default();
        is_writable = true;
    }

    Manager::new(
        Arc::new(connections),
        kind,
        write_is_sticky,
        sticky_duration,
        is_writable,
    )
}

pub async fn setup_pool_reslovers() {
    MariadbPoolManager::register().await;
    MysqlPoolManager::register().await;
    PostgresPoolManager::register().await;
    SqlitePoolManager::register().await;
}
