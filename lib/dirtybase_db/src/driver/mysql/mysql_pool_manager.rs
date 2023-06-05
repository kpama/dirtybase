use super::mysql_schema_manager::MySqlSchemaManager;
use crate::base::{
    connection::{ConnectionPoolRegisterTrait, ConnectionPoolTrait},
    schema::SchemaManagerTrait,
};
use async_trait::async_trait;
use sqlx::{any::AnyKind, mysql::MySqlPoolOptions, MySql, Pool};
use std::{str::FromStr, sync::Arc};

pub struct MySqlPoolManagerRegisterer;

#[async_trait]
impl ConnectionPoolRegisterTrait for MySqlPoolManagerRegisterer {
    async fn register(&self, conn_str: &str, max: u32) -> Option<Box<dyn ConnectionPoolTrait>> {
        if let Ok(kind) = AnyKind::from_str(conn_str) {
            if kind == AnyKind::MySql {
                return match db_connect(conn_str, max).await {
                    Ok(db_pool) => Some(Box::new(MysqlPoolManager {
                        db_pool: Arc::new(db_pool),
                    })),
                    Err(_) => None,
                };
            }
        }

        None
    }
}

#[derive(Debug)]
pub struct MysqlPoolManager {
    db_pool: Arc<Pool<MySql>>,
}

#[async_trait]
impl ConnectionPoolTrait for MysqlPoolManager {
    fn schema_manger(&self) -> Box<dyn SchemaManagerTrait> {
        Box::new(MySqlSchemaManager::new(self.db_pool.clone()))
    }
    fn id(&self) -> String {
        "mysql".into()
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
