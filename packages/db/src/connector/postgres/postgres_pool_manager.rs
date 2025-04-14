use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::{
    base::{
        connection::ConnectionPoolTrait,
        schema::{ClientType, DatabaseKind, SchemaManagerTrait},
    },
    config::{ConfigSet, ConnectionConfig},
    pool_manager_resolver::DbPoolManagerResolver,
};

use super::postgres_schema_manager::{POSTGRES_KIND, PostgresSchemaManager};

#[derive(Debug)]
pub struct PostgresPoolManager {
    db_pool: Arc<Pool<Postgres>>,
}

impl PostgresPoolManager {
    pub async fn register() {
        DbPoolManagerResolver::register(POSTGRES_KIND, |mut resolver| {
            Box::pin(async {
                resolver.set_pool_manager(resolve(resolver.config_ref()).await);
                resolver
            })
        })
        .await;
    }
}

#[async_trait]
impl ConnectionPoolTrait for PostgresPoolManager {
    fn schema_manger(&self) -> Box<dyn SchemaManagerTrait + Send + Sync> {
        Box::new(PostgresSchemaManager::new(self.db_pool.clone()))
    }
    fn id(&self) -> DatabaseKind {
        POSTGRES_KIND.into()
    }

    async fn close(&self) {
        self.db_pool.close().await;
    }
}

pub async fn resolve(
    config_set: &ConfigSet,
) -> Result<HashMap<ClientType, Box<dyn ConnectionPoolTrait>>, anyhow::Error> {
    let mut pools: HashMap<ClientType, Box<dyn ConnectionPoolTrait>> = HashMap::new();
    for (client_type, config) in config_set.iter() {
        if POSTGRES_KIND == config.kind_ref().as_str() && config.enable {
            match db_connect(config).await {
                Ok(db_pool) => {
                    pools.insert(
                        *client_type,
                        Box::new(PostgresPoolManager {
                            db_pool: Arc::new(db_pool),
                        }),
                    );
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    if pools.is_empty() {
        Err(anyhow!("could not create any pool manager for postgres"))
    } else {
        Ok(pools)
    }
}

pub async fn db_connect(config: &ConnectionConfig) -> anyhow::Result<Pool<Postgres>> {
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
