#![allow(dead_code)]

use super::base::schema::DatabaseKind;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct BaseConfig {
    pub enable: bool,
    pub url: String,
    pub max: u32,
    pub sticky: Option<bool>,
    pub sticky_duration: Option<i64>,
    pub foreign_key: Option<bool>,
    pub busy_timeout: Option<u64>,
}

impl Default for BaseConfig {
    /// By default the data is sqlite and
    /// the database is in memory
    fn default() -> Self {
        Self {
            enable: true,
            url: "sqlite::memory:".to_string(),
            max: 2,
            sticky: Some(true),
            sticky_duration: Some(10),
            foreign_key: Some(true),
            busy_timeout: Some(60),
        }
    }
}

#[derive(Debug, serde::Deserialize, Clone, Default)]
pub struct DirtybaseDbConfig {
    pub default: Option<DatabaseKind>,
    pub mysql_read: Option<BaseConfig>,
    pub mysql_write: Option<BaseConfig>,
    pub postgres_read: Option<BaseConfig>,
    pub postgres_write: Option<BaseConfig>,
    pub sqlite_read: Option<BaseConfig>,
    pub sqlite_write: Option<BaseConfig>,
}

impl DirtybaseDbConfig {
    pub async fn new(config: &dirtybase_config::DirtyConfig) -> Self {
        config
            .optional_file("database.toml", Some("DTY_DB"))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }

    pub fn in_memory() -> Self {
        Self::new_sqlite_write(BaseConfig::default())
    }

    pub fn new_mysql_write(mut write_config: BaseConfig) -> Self {
        write_config.enable = true;
        Self {
            default: Some(DatabaseKind::Mysql),
            mysql_write: Some(write_config),
            ..Default::default()
        }
    }

    pub fn new_mysql_read(mut read_config: BaseConfig) -> Self {
        read_config.enable = true;
        Self {
            default: Some(DatabaseKind::Mysql),
            mysql_read: Some(read_config),
            ..Default::default()
        }
    }

    pub fn new_mysql(mut write_config: BaseConfig, read_config: Option<BaseConfig>) -> Self {
        write_config.enable = true;
        Self {
            default: Some(DatabaseKind::Mysql),
            mysql_write: Some(write_config),
            mysql_read: read_config,
            ..Default::default()
        }
    }

    pub fn new_sqlite_write(mut write_config: BaseConfig) -> Self {
        write_config.enable = true;
        Self {
            default: Some(DatabaseKind::Sqlite),
            sqlite_write: Some(write_config),
            ..Default::default()
        }
    }

    pub fn new_sqlite_read(mut read_config: BaseConfig) -> Self {
        read_config.enable = true;
        Self {
            default: Some(DatabaseKind::Sqlite),
            sqlite_read: Some(read_config),
            ..Default::default()
        }
    }

    pub fn new_sqlite(write_config: BaseConfig, read_config: Option<BaseConfig>) -> Self {
        Self {
            default: Some(DatabaseKind::Sqlite),
            sqlite_write: Some(write_config),
            sqlite_read: read_config,
            ..Default::default()
        }
    }

    pub fn new_postgres_write(mut write_config: BaseConfig) -> Self {
        write_config.enable = true;
        Self {
            default: Some(DatabaseKind::Postgres),
            postgres_write: Some(write_config),
            ..Default::default()
        }
    }

    pub fn new_postgres_read(mut read_config: BaseConfig) -> Self {
        read_config.enable = true;
        Self {
            default: Some(DatabaseKind::Postgres),
            postgres_read: Some(read_config),
            ..Default::default()
        }
    }

    pub fn new_postgres(write_config: BaseConfig, read_config: Option<BaseConfig>) -> Self {
        Self {
            default: Some(DatabaseKind::Postgres),
            postgres_write: Some(write_config),
            postgres_read: read_config,
            ..Default::default()
        }
    }
}
