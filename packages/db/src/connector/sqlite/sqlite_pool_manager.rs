use std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration};

use crate::{
    ConnectionPoolRegisterTrait,
    base::{
        connection::ConnectionPoolTrait,
        schema::{ClientType, DatabaseKind},
    },
    config::{BaseConfig, ConfigSet},
};
use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::{
    Pool, Sqlite,
    sqlite::SqliteJournalMode,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

use super::sqlite_schema_manager::{SQLITE_KIND, SqliteSchemaManager};

pub struct SqlitePoolManagerRegisterer;

#[async_trait]
impl ConnectionPoolRegisterTrait for SqlitePoolManagerRegisterer {
    async fn register(
        &self,
        config_set: &ConfigSet,
    ) -> Result<HashMap<ClientType, Box<dyn ConnectionPoolTrait>>, anyhow::Error> {
        let mut pools: HashMap<ClientType, Box<dyn ConnectionPoolTrait>> = HashMap::new();
        for (client_type, config) in config_set.iter() {
            if SQLITE_KIND == config.kind().as_str() && config.enable {
                match db_connect(config).await {
                    Ok(db_pool) => {
                        pools.insert(
                            *client_type,
                            Box::new(SqlitePoolManager {
                                db_pool: Arc::new(db_pool),
                            }),
                        );
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        if pools.is_empty() {
            Err(anyhow!("could not create any pool manager for sqlite"))
        } else {
            Ok(pools)
        }
    }
}

#[derive(Debug)]
pub struct SqlitePoolManager {
    db_pool: Arc<Pool<Sqlite>>,
}

#[async_trait]
impl ConnectionPoolTrait for SqlitePoolManager {
    fn schema_manger(&self) -> Box<dyn crate::base::schema::SchemaManagerTrait + Send + Sync> {
        Box::new(SqliteSchemaManager::new(self.db_pool.clone()))
    }

    fn id(&self) -> DatabaseKind {
        "sqlite".into()
    }

    async fn close(&self) {
        self.db_pool.close().await;
    }
}

pub async fn db_connect(config: &BaseConfig) -> anyhow::Result<Pool<Sqlite>> {
    let mut option = SqliteConnectOptions::from_str(&config.url)
        .unwrap()
        .foreign_keys(true)
        .create_if_missing(true);

    if config.client_type == ClientType::Write {
        option = option
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(config.busy_timeout.unwrap_or(5)));
    } else {
        option = option.read_only(true);
    }

    match SqlitePoolOptions::new()
        .max_connections(config.max)
        .connect_with(option)
        .await
    {
        Ok(conn) => {
            log::info!("Maximum DB pool connection: {}", config.max);
            Ok(conn)
        }
        Err(e) => {
            // TODO: Use i18n
            log::error!("could not connect to the database: {:#?}", &e);
            Err(anyhow!(e))
        }
    }
}
