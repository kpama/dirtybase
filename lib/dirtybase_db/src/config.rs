#![allow(dead_code)]

use crate::base::schema::DatabaseKind;
#[derive(Debug, serde::Deserialize, Clone)]
pub struct BaseConfig {
    pub(crate) enable: bool,
    pub(crate) url: String,
    pub(crate) max: u32,
    pub(crate) sticky: Option<bool>,
    pub(crate) sticky_duration: Option<i64>,
    pub(crate) foreign_key: Option<bool>,
    pub(crate) busy_timeout: Option<u64>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct DirtybaseDbConfig {
    pub(crate) default: Option<DatabaseKind>,
    pub(crate) mysql_read: Option<BaseConfig>,
    pub(crate) mysql_write: Option<BaseConfig>,
    pub(crate) postgres_read: Option<BaseConfig>,
    pub(crate) postgres_write: Option<BaseConfig>,
    pub(crate) sqlite_read: Option<BaseConfig>,
    pub(crate) sqlite_write: Option<BaseConfig>,
}
