use super::mysql_schema_manager::MySqlSchemaManager;
use async_trait::async_trait;
use dirtybase_contract::db::{
    base::{
        connection::{ConnectionPoolRegisterTrait, ConnectionPoolTrait},
        schema::{ClientType, DatabaseKind, SchemaManagerTrait},
    },
    config::{BaseConfig, DirtybaseDbConfig},
};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use std::{collections::HashMap, sync::Arc};

pub struct MySqlPoolManagerRegisterer;

#[async_trait]
impl ConnectionPoolRegisterTrait for MySqlPoolManagerRegisterer {
    async fn register(
        &self,
        config: &DirtybaseDbConfig,
    ) -> Option<HashMap<ClientType, Box<dyn ConnectionPoolTrait>>> {
        let mut pools: HashMap<ClientType, Box<dyn ConnectionPoolTrait>> = HashMap::new();

        // read pool
        if let Some(read_config) = &config.mysql_read {
            if read_config.enable {
                if let Ok(db_pool) = db_connect(read_config).await {
                    pools.insert(
                        ClientType::Read,
                        Box::new(MysqlPoolManager {
                            db_pool: Arc::new(db_pool),
                        }),
                    );
                }
            }
        }

        // write pool
        if let Some(write_config) = &config.mysql_write {
            if write_config.enable {
                if let Ok(db_pool) = db_connect(write_config).await {
                    pools.insert(
                        ClientType::Write,
                        Box::new(MysqlPoolManager {
                            db_pool: Arc::new(db_pool),
                        }),
                    );
                }
            }
        }

        if pools.is_empty() {
            return None;
        } else {
            return Some(pools);
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

pub async fn db_connect(config: &BaseConfig) -> anyhow::Result<Pool<MySql>> {
    match MySqlPoolOptions::new()
        .max_connections(config.max)
        .connect(&config.url)
        .await
    {
        Ok(conn) => {
            log::info!("MySql maximum DB pool connection: {}", config.max);
            Ok(conn)
        }
        Err(e) => {
            log::error!("could not connect to the MySql: {:#?}", &e);
            Err(anyhow::anyhow!("could not connect to the MySql: {:#?}", e))
        }
    }
}
