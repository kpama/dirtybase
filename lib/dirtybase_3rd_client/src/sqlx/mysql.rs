use std::sync::Arc;

use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use tokio::sync::OnceCell;

type ConnectionResult = Arc<Pool<MySql>>;

static DTY_SQLX_MYSQL_POOL: OnceCell<ConnectionResult> = OnceCell::const_new();

pub(crate) async fn db_connect(conn: &str, max_connection: u32) -> ConnectionResult {
    DTY_SQLX_MYSQL_POOL
        .get_or_init(|| async {
            match MySqlPoolOptions::new()
                .max_connections(max_connection)
                .connect(conn)
                .await
            {
                Ok(conn) => {
                    log::info!("Maximum DB pool connection: {}", max_connection);
                    Arc::new(conn)
                }
                Err(e) => {
                    log::error!("could not connect to the database: {:#?}", &e);
                    panic!("could not connect to the database: {:#?}", e);
                }
            }
        })
        .await
        .clone()
}

async fn foo() {
    let x = db_connect("dfd", 55).await;
}
