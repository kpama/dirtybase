use std::{str::FromStr, sync::Arc};

use crate::base::{
    connection::{ConnectionPoolRegisterTrait, ConnectionPoolTrait},
    schema::DatabaseKind,
};
use async_trait::async_trait;
use sqlx::{
    sqlite::SqliteJournalMode,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};

use super::sqlite_schema_manager::SqliteSchemaManager;

pub struct SqlitePoolManagerRegisterer;

#[async_trait]
impl ConnectionPoolRegisterTrait for SqlitePoolManagerRegisterer {
    async fn register(&self, conn_str: &str, max: u32) -> Option<Box<dyn ConnectionPoolTrait>> {
        if conn_str.starts_with("sqlite:") {
            return match db_connect(conn_str, max).await {
                Ok(db_pool) => Some(Box::new(SqlitePoolManager {
                    db_pool: Arc::new(db_pool),
                })),
                Err(_) => None,
            };
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct SqlitePoolManager {
    db_pool: Arc<Pool<Sqlite>>,
}

impl ConnectionPoolTrait for SqlitePoolManager {
    fn schema_manger(&self) -> Box<dyn crate::base::schema::SchemaManagerTrait + Send + Sync> {
        Box::new(SqliteSchemaManager::new(self.db_pool.clone()))
    }

    fn id(&self) -> DatabaseKind {
        DatabaseKind::Sqlite
    }
}

pub async fn db_connect(conn: &str, max_connection: u32) -> anyhow::Result<Pool<Sqlite>> {
    match SqlitePoolOptions::new()
        .max_connections(max_connection)
        .connect_with(
            SqliteConnectOptions::from_str(conn)
                .unwrap()
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal),
        )
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
