#![allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
pub struct BaseConfig {
    enable: bool,
    url: String,
    max: u32,
    sticky: bool,
    sticky_duration: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct SqliteConfig {
    enable: bool,
    url: String,
    max: u32,
    foreign_key: bool,
    busy_timeout: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct DirtybaseDbConfig {
    mysql_read: Option<BaseConfig>,
    mysql_write: Option<BaseConfig>,
    postgres_read: Option<BaseConfig>,
    postgres_write: Option<BaseConfig>,
    sqlite_read: Option<SqliteConfig>,
    sqlite_write: Option<SqliteConfig>,
}
