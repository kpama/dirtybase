#![allow(dead_code)]

use super::base::schema::DatabaseKind;

#[derive(Debug, Default, serde::Deserialize, Clone)]
pub struct BaseConfig {
    pub enable: bool,
    pub url: String,
    pub max: u32,
    pub sticky: Option<bool>,
    pub sticky_duration: Option<i64>,
    pub foreign_key: Option<bool>,
    pub busy_timeout: Option<u64>,
}

#[derive(Debug, serde::Deserialize, Clone)]
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
}
