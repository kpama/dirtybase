use std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration};

use crate::{
    base::{
        connection::{ConnectionPoolRegisterTrait, ConnectionPoolTrait},
        schema::{ClientType, DatabaseKind},
    },
    config::{BaseConfig, DirtybaseDbConfig},
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
    async fn register(
        &self,
        config: &DirtybaseDbConfig,
    ) -> Option<HashMap<ClientType, Box<dyn ConnectionPoolTrait>>> {
        let mut pools: HashMap<ClientType, Box<dyn ConnectionPoolTrait>> = HashMap::new();

        // read pool
        if let Some(read_config) = &config.sqlite_read {
            if read_config.enable {
                if let Ok(db_pool) = db_connect(read_config, false).await {
                    pools.insert(
                        ClientType::Read,
                        Box::new(SqlitePoolManager {
                            db_pool: Arc::new(db_pool),
                        }),
                    );
                }
            }
        }

        // write pool
        if let Some(write_config) = &config.sqlite_write {
            if write_config.enable {
                if let Ok(db_pool) = db_connect(write_config, true).await {
                    pools.insert(
                        ClientType::Write,
                        Box::new(SqlitePoolManager {
                            db_pool: Arc::new(db_pool),
                        }),
                    );
                }
            }
        }

        if pools.is_empty() {
            None
        } else {
            Some(pools)
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

pub async fn db_connect(config: &BaseConfig, for_write: bool) -> anyhow::Result<Pool<Sqlite>> {
    let mut option = SqliteConnectOptions::from_str(&config.url)
        .unwrap()
        .foreign_keys(true)
        .create_if_missing(true);

    if for_write {
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
            panic!("could not connect to the database: {:#?}", &e);
        }
    }
}
