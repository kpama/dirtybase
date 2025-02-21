use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::{MySql, Pool, mysql::MySqlPoolOptions};
use std::{collections::HashMap, sync::Arc};

use crate::{
    ConnectionPoolRegisterTrait,
    base::{
        connection::ConnectionPoolTrait,
        schema::{ClientType, DatabaseKind, SchemaManagerTrait},
    },
    config::{BaseConfig, ConfigSet},
};

use super::mysql_schema_manager::{MYSQL_KIND, MySqlSchemaManager};

pub struct MySqlPoolManagerRegisterer;

#[async_trait]
impl ConnectionPoolRegisterTrait for MySqlPoolManagerRegisterer {
    async fn register(
        &self,
        config_set: &ConfigSet,
    ) -> Result<HashMap<ClientType, Box<dyn ConnectionPoolTrait>>, anyhow::Error> {
        let mut pools: HashMap<ClientType, Box<dyn ConnectionPoolTrait>> = HashMap::new();
        for (client_type, config) in config_set.iter() {
            if MYSQL_KIND == config.kind_ref().as_str() && config.enable {
                match db_connect(config).await {
                    Ok(db_pool) => {
                        pools.insert(
                            *client_type,
                            Box::new(MysqlPoolManager {
                                db_pool: Arc::new(db_pool),
                            }),
                        );
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        if pools.is_empty() {
            Err(anyhow!("could not create any pool manager for mysql"))
        } else {
            Ok(pools)
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
        MYSQL_KIND.into()
    }
    async fn close(&self) {
        self.db_pool.close().await;
    }
}

pub async fn db_connect(config: &BaseConfig) -> anyhow::Result<Pool<MySql>> {
    match MySqlPoolOptions::new()
        .max_connections(config.max)
        .connect(&config.url)
        .await
    {
        Ok(conn) => {
            // TODO: Use i18n
            log::info!("MySql maximum DB pool connection: {}", config.max);
            Ok(conn)
        }
        Err(e) => {
            // TODO: Use i18n
            log::error!("could not connect to mysql/mariadb: {:?}", e);
            Err(anyhow!(e))
        }
    }
}
