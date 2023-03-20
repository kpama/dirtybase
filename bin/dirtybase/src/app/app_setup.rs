use dirtybase_db::base::schema::SchemaManagerTrait;
use dirtybase_db::{base, driver::mysql::mysql_schema_manager::MySqlSchemaManager};
use sqlx::{any::AnyKind, mysql::MySqlPoolOptions, MySql, Pool};
use std::{str::FromStr, sync::Arc};
use super::setup_database::create_data_tables;

pub struct Dirtybase {
    db_pool: Arc<Pool<MySql>>,
    kind: AnyKind,
}

impl Dirtybase {
    pub async fn new(db_connection: &str, db_max_connection: u32) -> anyhow::Result<Self> {
        let kind = AnyKind::from_str(db_connection).unwrap_or(AnyKind::MySql);
        let instance = Self {
            kind,
            db_pool: Arc::new(db_connect(db_connection, db_max_connection).await),
        };

        // match instance.kind {
        //     // @todo implement the other supported databases' driver
        //     _ => instance.mysql_pool = Some(Arc::new(db_connect(&instance.url).await)),
        // };

        Ok(instance)
    }

    pub fn kind(&self) -> &AnyKind {
        &self.kind
    }

    pub fn schema_manger(&self) -> base::manager::Manager {
        // TODO Check the database `kind`
        // match self.kind {
        //     _ => base::manager::Manager::new(Box::new(MySqlSchemaManager::instance(
        //         self.db_pool.clone(),
        //     ))),
        // }

        base::manager::Manager::new(Box::new(MySqlSchemaManager::instance(self.db_pool.clone())))
    }

    pub async fn db_setup(&self) {
        create_data_tables(self.schema_manger()).await;
    }
}

pub async fn db_connect(conn: &str, max_connection: u32) -> Pool<MySql> {
    match MySqlPoolOptions::new()
        .max_connections(max_connection)
        .connect(conn)
        .await
    {
        Ok(conn) => {
            log::info!("Maximum DB pool connection: {}", max_connection);
            conn
        }
        Err(e) => {
            log::error!("could not connect to the database: {:#?}", &e);
            panic!("could not connect to the database: {:#?}", e);
        }
    }
}
