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
    config::{ConfigSet, ConnectionConfig},
    connector::mariadb::LOG_TARGET,
};

use super::mariadb_schema_manager::{MARIADB_KIND, MariadbSchemaManager};

pub struct MariadbPoolManagerRegisterer;

#[async_trait]
impl ConnectionPoolRegisterTrait for MariadbPoolManagerRegisterer {
    async fn register(
        &self,
        config_set: &ConfigSet,
    ) -> Result<HashMap<ClientType, Box<dyn ConnectionPoolTrait>>, anyhow::Error> {
        let mut pools: HashMap<ClientType, Box<dyn ConnectionPoolTrait>> = HashMap::new();
        for (client_type, config) in config_set.iter() {
            if MARIADB_KIND == config.kind_ref().as_str() && config.enable {
                match db_connect(config).await {
                    Ok(db_pool) => {
                        pools.insert(
                            *client_type,
                            Box::new(MariadbPoolManager {
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
pub struct MariadbPoolManager {
    db_pool: Arc<Pool<MySql>>,
}

#[async_trait]
impl ConnectionPoolTrait for MariadbPoolManager {
    fn schema_manger(&self) -> Box<dyn SchemaManagerTrait + Send + Sync> {
        Box::new(MariadbSchemaManager::new(self.db_pool.clone()))
    }
    fn id(&self) -> DatabaseKind {
        MARIADB_KIND.into()
    }
    async fn close(&self) {
        self.db_pool.close().await;
    }
}

pub async fn db_connect(config: &ConnectionConfig) -> anyhow::Result<Pool<MySql>> {
    match MySqlPoolOptions::new()
        .max_connections(config.max)
        .connect(&config.url)
        .await
    {
        Ok(conn) => {
            // TODO: Use i18n
            log::info!(target: LOG_TARGET,"maximum DB pool connection: {}", config.max);
            Ok(conn)
        }
        Err(e) => {
            // TODO: Use i18n
            log::error!(target: LOG_TARGET,"could not connect to mariadb: {:?}", e);
            Err(anyhow!(e))
        }
    }
}
