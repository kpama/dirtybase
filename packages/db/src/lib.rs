pub mod connector;
mod dirtybase_entry;

use base::{connection::ConnectionPoolRegisterTrait, schema::DatabaseKind};
use connection_bus::MakePoolManagerCommand;
use connector::{
    mysql::mysql_pool_manager::MySqlPoolManagerRegisterer,
    postgres::postgres_pool_manager::PostgresPoolManagerRegisterer,
    sqlite::sqlite_pool_manager::SqlitePoolManagerRegisterer,
};
pub use dirtybase_contract::db::*;
pub use dirtybase_entry::*;

use busstop::DispatchableQuery;

pub use anyhow;

pub const USER_TABLE: &str = "core_user";

pub async fn setup_handlers() {
    MakePoolManagerCommand::query_middleware(|dispatched, next| {
        Box::pin(async {
            if let Some(query) = dispatched.the_query::<MakePoolManagerCommand>() {
                match query.kind() {
                    DatabaseKind::Mysql => {
                        let mysql_pool_registerer = MySqlPoolManagerRegisterer;
                        let r = mysql_pool_registerer.register(query.config_set_ref()).await;
                        query.set_result(&dispatched, r);
                        return dispatched;
                    }
                    DatabaseKind::Postgres => {
                        let postgres_pool_registerer = PostgresPoolManagerRegisterer;
                        let r = postgres_pool_registerer
                            .register(query.config_set_ref())
                            .await;
                        query.set_result(&dispatched, r);
                        return dispatched;
                    }
                    DatabaseKind::Sqlite => {
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
