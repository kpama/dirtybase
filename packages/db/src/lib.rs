pub mod config;
pub mod connection_bus;
pub mod connector;
mod dirtybase_entry;
mod resource_manager;

use std::{collections::HashMap, sync::Arc};

use base::schema::DatabaseKind;
use config::ConfigSet;
use connection_bus::MakePoolManagerCommand;
use connector::{
    mysql::{mysql_pool_manager::MySqlPoolManagerRegisterer, mysql_schema_manager::MYSQL_KIND},
    postgres::{
        postgres_pool_manager::PostgresPoolManagerRegisterer,
        postgres_schema_manager::POSTGRES_KIND,
    },
    sqlite::{
        sqlite_pool_manager::SqlitePoolManagerRegisterer, sqlite_schema_manager::SQLITE_KIND,
    },
};
use dirtybase_contract::db::base::{
    connection::ConnectionPoolTrait, manager::Manager, schema::ClientType,
};
pub use dirtybase_contract::db::*;
pub use dirtybase_entry::*;

use busstop::DispatchableQuery;

pub use anyhow;

pub const USER_TABLE: &str = "core_user";

#[async_trait::async_trait]
pub trait ConnectionPoolRegisterTrait: Send {
    async fn register(
        &self,
        config: &ConfigSet,
    ) -> Result<HashMap<ClientType, Box<dyn ConnectionPoolTrait>>, anyhow::Error>;
}

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

pub async fn setup_handlers() {
    MakePoolManagerCommand::query_middleware(|dispatched, next| {
        Box::pin(async {
            if let Some(query) = dispatched.the_query::<MakePoolManagerCommand>() {
                match query.kind().as_str() {
                    MYSQL_KIND => {
                        let mysql_pool_registerer = MySqlPoolManagerRegisterer;
                        let r = mysql_pool_registerer.register(query.config_set_ref()).await;
                        query.set_result(&dispatched, r);
                        return dispatched;
                    }
                    POSTGRES_KIND => {
                        let postgres_pool_registerer = PostgresPoolManagerRegisterer;
                        let r = postgres_pool_registerer
                            .register(query.config_set_ref())
                            .await;
                        query.set_result(&dispatched, r);
                        return dispatched;
                    }
                    SQLITE_KIND => {
                        let sqlite_pool_registerer = SqlitePoolManagerRegisterer;
                        let r = sqlite_pool_registerer
                            .register(query.config_set_ref())
                            .await;
                        query.set_result(&dispatched, r);
                        return dispatched;
                    }
                    _ => return next.call(dispatched).await,
                }
            }
            next.call(dispatched).await
        })
    })
    .await;
}
