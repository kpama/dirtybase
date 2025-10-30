use std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration};

use crate::{
    base::{
        connection::ConnectionPoolTrait,
        schema::{ClientType, DatabaseKind},
    },
    config::{ConfigSet, ConnectionConfig},
    connector::sqlite::LOG_TARGET,
    pool_manager_resolver::DbPoolManagerResolver,
};
use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::{
    Pool, Sqlite,
    sqlite::SqliteJournalMode,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

use super::sqlite_connector::{SQLITE_KIND, SqliteSchemaManager};

#[derive(Debug)]
pub struct SqlitePoolManager {
    db_pool: Arc<Pool<Sqlite>>,
}

impl SqlitePoolManager {
    pub async fn register() {
        DbPoolManagerResolver::register(SQLITE_KIND, |mut resolver| {
            Box::pin(async {
                resolver.set_pool_manager(resolve(resolver.config_ref()).await);
                resolver
            })
        })
        .await;
    }
}

#[async_trait]
impl ConnectionPoolTrait for SqlitePoolManager {
    fn schema_manger(&self) -> Box<dyn crate::base::schema::SchemaManagerTrait + Send + Sync> {
        Box::new(SqliteSchemaManager::new(self.db_pool.clone()))
    }

    fn id(&self) -> DatabaseKind {
        SQLITE_KIND.into()
    }

    async fn close(&self) {
        if !self.db_pool.is_closed() {
            self.db_pool.close().await;
        }
        tracing::trace!(target: LOG_TARGET,"connection closed: {}", self.db_pool.is_closed());
    }
}

pub async fn resolve(
    config_set: &ConfigSet,
) -> Result<HashMap<ClientType, Box<dyn ConnectionPoolTrait>>, anyhow::Error> {
    let mut pools: HashMap<ClientType, Box<dyn ConnectionPoolTrait>> = HashMap::new();
    for (client_type, config) in config_set.iter() {
        if SQLITE_KIND == config.kind_ref().as_str() && config.enable {
            match db_connect(config).await {
                Ok(db_pool) => {
                    pools.insert(
                        *client_type,
                        Box::new(SqlitePoolManager {
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
        Err(anyhow!("could not create any pool manager for mariadb"))
    } else {
        Ok(pools)
    }
}

pub async fn db_connect(config: &ConnectionConfig) -> anyhow::Result<Pool<Sqlite>> {
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
            tracing::debug!(target:LOG_TARGET,"maximum DB pool connection: {}", config.max);
            Ok(conn)
        }
        Err(e) => {
            // TODO: Use i18n
            tracing::error!("could not connect to the database: {:#?}", &e);
            Err(anyhow!(e))
        }
    }
}
