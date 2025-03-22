use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use dirtybase_contract::{
    app::Context,
    config::{DirtyConfig, TryFromDirtyConfig},
};

use crate::connector::sqlite::sqlite_schema_manager::SQLITE_KIND;

use super::base::schema::{ClientType, DatabaseKind};

pub type ConfigSet = HashMap<ClientType, ConnectionConfig>;
pub type ConfigCollection = HashMap<String, ConfigSet>;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct DbConfig {
    #[serde(default)]
    enable: bool,
    #[serde(default)]
    idle_timeout: i64,
    #[serde(default)]
    default: Arc<String>,
    #[serde(alias = "clients")]
    collection: Arc<ConfigCollection>,
}

#[async_trait::async_trait]
impl TryFromDirtyConfig for DbConfig {
    type Returns = Self;
    async fn from_config(
        config: &DirtyConfig,
        _ctx: &Context,
    ) -> Result<Self::Returns, anyhow::Error> {
        config
            .optional_file("database.toml", Some("DTY_DB"))
            .build()
            .await?
            .try_deserialize::<Self>()
            .map_err(|e| anyhow!(e.to_string()))
    }
}

impl DbConfig {
    pub fn is_enable(&self) -> bool {
        self.enable
    }

    pub fn idle_timeout(&self) -> i64 {
        self.idle_timeout
    }

    pub fn default_set(&self) -> Option<ConfigSet> {
        if self.default.is_empty() || !self.enable {
            return None;
        }

        self.collection.get(self.default.as_str()).cloned()
    }

    pub fn get_set(&self, name: &str) -> Option<ConfigSet> {
        self.collection.get(name).cloned()
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ConnectionConfig {
    #[serde(default)]
    pub enable: bool,
    pub kind: DatabaseKind,
    #[serde(default)]
    pub client_type: ClientType,
    pub url: String,
    pub max: u32,
    pub sticky: Option<bool>,
    pub sticky_duration: Option<i64>,
    pub foreign_key: Option<bool>,
    pub busy_timeout: Option<u64>,
    pub custom: Option<HashMap<String, String>>,
}

/// By default the data is sqlite and
/// the database is in memory
impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            enable: true,
            kind: SQLITE_KIND.into(),
            client_type: ClientType::Write,
            url: "sqlite::memory:".to_string(),
            max: 2,
            sticky: Some(true),
            sticky_duration: Some(10),
            foreign_key: Some(true),
            busy_timeout: Some(60),
            custom: None,
        }
    }
}

impl ConnectionConfig {
    pub fn kind(&self) -> DatabaseKind {
        self.kind.clone()
    }

    pub fn kind_ref(&self) -> &DatabaseKind {
        &self.kind
    }

    pub async fn new_set() -> ConfigSet {
        return Self::set_from(&DirtyConfig::new()).await;
    }

    pub async fn set_from(dirty_config: &DirtyConfig) -> ConfigSet {
        let config = dirty_config
            .optional_file("database.toml", Some("DTY_DB"))
            .build()
            .await
            .expect("could not load the database configuration");

        let collection = config
            .get::<ConfigCollection>("clients")
            .expect("could not parse the data configuration");

        let default = config.get::<String>("default").unwrap();
        collection.get(&default).unwrap().clone()
    }

    pub fn in_memory_set() -> ConfigSet {
        let mut set = ConfigSet::new();
        set.insert(ClientType::Write, Self::default());

        set
    }
}
