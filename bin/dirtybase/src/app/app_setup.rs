use super::setup_database::create_data_tables;
use dirtybase_db::base::schema::GraphDbClient;
use dirtybase_db::base::schema::GraphDbTrait;
use dirtybase_db::base::schema::RelationalDbTrait;
use dirtybase_db::base::schema::SchemaManagerTrait;
use dirtybase_db::driver::surreal::surreal_schema_manager::SurrealGraphDbClient;
use dirtybase_db::driver::surreal::surreal_schema_manager::SurrealSchemaManager;
use dirtybase_db::driver::surreal::{SurrealClient, SurrealDbConfig};
use dirtybase_db::{base, driver::mysql::mysql_schema_manager::MySqlSchemaManager};
use sqlx::{any::AnyKind, mysql::MySqlPoolOptions, MySql, Pool};
use std::env;
use std::sync::Mutex;
use std::{str::FromStr, sync::Arc};

pub struct DirtyBase {
    db_pool: Arc<Pool<MySql>>,
    graph_client: Arc<Mutex<dyn GraphDbClient>>,
    kind: AnyKind,
}

impl DirtyBase {
    pub async fn new(
        db_connection: &str,
        db_max_connection: u32,
        surreal_config: SurrealDbConfig,
    ) -> anyhow::Result<Self> {
        let kind = AnyKind::from_str(db_connection).unwrap_or(AnyKind::MySql);
        let surreal_client = SurrealGraphDbClient::new(Arc::new(
            dirtybase_db::driver::surreal::setup(surreal_config).await,
        ));
        let instance = Self {
            kind,
            db_pool: Arc::new(db_connect(db_connection, db_max_connection).await?),
            graph_client: Arc::new(Mutex::new(surreal_client)),
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

        let db = MySqlSchemaManager::instance(self.db_pool.clone());
        base::manager::Manager::new(Box::new(db))
    }

    pub fn graphdb_schema_manager(&self) -> base::manager::Manager {
        let db = SurrealSchemaManager::instance(
            self.graph_client
                .clone()
                .lock()
                .unwrap()
                .into_inner_client(),
        );
        log::info!("surreal schema manager created");
        base::manager::Manager::new(Box::new(db))
    }

    pub async fn db_setup(&self) {
        create_data_tables(self.schema_manger()).await;
    }
}

pub async fn db_connect(conn: &str, max_connection: u32) -> anyhow::Result<Pool<MySql>> {
    match MySqlPoolOptions::new()
        .max_connections(max_connection)
        .connect(conn)
        .await
    {
        Ok(conn) => {
            log::info!("Maximum DB pool connection: {}", max_connection);
            Ok(conn)
        }
        Err(e) => {
            log::error!("could not connect to the database: {:#?}", &e);
            Err(anyhow::anyhow!(
                "could not connect to the database: {:#?}",
                e
            ))
        }
    }
}
