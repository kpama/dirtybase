use super::mysql_schema_manager::MySqlSchemaManager;
use crate::base::{
    connection::{ConnectionPoolRegisterTrait, ConnectionPoolTrait},
    schema::{DatabaseKind, SchemaManagerTrait},
};
use async_trait::async_trait;
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use std::sync::Arc;

pub struct MySqlPoolManagerRegisterer;

#[async_trait]
impl ConnectionPoolRegisterTrait for MySqlPoolManagerRegisterer {
    async fn register(&self, conn_str: &str, max: u32) -> Option<Box<dyn ConnectionPoolTrait>> {
        if conn_str.starts_with("mysql:") || conn_str.starts_with("mariadb:") {
            return match db_connect(conn_str, max).await {
                Ok(db_pool) => Some(Box::new(MysqlPoolManager {
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
pub struct MysqlPoolManager {
    db_pool: Arc<Pool<MySql>>,
}

#[async_trait]
impl ConnectionPoolTrait for MysqlPoolManager {
    fn schema_manger(&self) -> Box<dyn SchemaManagerTrait + Send + Sync> {
        Box::new(MySqlSchemaManager::new(self.db_pool.clone()))
    }
    fn id(&self) -> DatabaseKind {
        DatabaseKind::Mysql
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
