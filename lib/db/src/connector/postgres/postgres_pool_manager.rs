use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use async_trait::async_trait;
use dirtybase_contract::db::config::ConfigSet;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::{
    base::{
        connection::{ConnectionPoolRegisterTrait, ConnectionPoolTrait},
        schema::{ClientType, DatabaseKind, SchemaManagerTrait},
    },
    config::BaseConfig,
};

use super::postgres_schema_manager::PostgresSchemaManager;

pub struct PostgresPoolManagerRegisterer;

#[async_trait]
impl ConnectionPoolRegisterTrait for PostgresPoolManagerRegisterer {
    async fn register(
        &self,
        config_set: &ConfigSet,
    ) -> Result<HashMap<ClientType, Box<dyn ConnectionPoolTrait>>, anyhow::Error> {
        let mut pools: HashMap<ClientType, Box<dyn ConnectionPoolTrait>> = HashMap::new();
        for (client_type, config) in config_set.iter() {
            if config.kind() == DatabaseKind::Postgres {
                match db_connect(config).await {
                    Ok(db_pool) => {
                        pools.insert(
                            client_type.clone(),
                            Box::new(PostgresPoolManager {
                                db_pool: Arc::new(db_pool),
                            }),
                        );
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        if pools.is_empty() {
            Err(anyhow!(
                "could not create any pool manager for kind: {:?}",
                &DatabaseKind::Postgres
            ))
        } else {
            Ok(pools)
        }
    }
}

#[derive(Debug)]
pub struct PostgresPoolManager {
    db_pool: Arc<Pool<Postgres>>,
}

#[async_trait]
impl ConnectionPoolTrait for PostgresPoolManager {
    fn schema_manger(&self) -> Box<dyn SchemaManagerTrait + Send + Sync> {
        Box::new(PostgresSchemaManager::new(self.db_pool.clone()))
    }
    fn id(&self) -> DatabaseKind {
        DatabaseKind::Postgres
    }
}

pub async fn db_connect(config: &BaseConfig) -> anyhow::Result<Pool<Postgres>> {
    match PgPoolOptions::new()
        .max_connections(config.max)
        .connect(&config.url)
        .await
    {
        Ok(conn) => {
            log::info!("Postgres maximum DB pool connection: {}", config.max);
            Ok(conn)
        }
        Err(e) => {
            // TODO: Use i18n
            log::error!("could not connect to postgres: {:#?}", &e);
            Err(anyhow!(e))
        }
    }
}
