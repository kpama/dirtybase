use std::collections::HashMap;

use crate::config::DirtyConfig;

use super::base::schema::{ClientType, DatabaseKind};

pub type ConfigSet = HashMap<ClientType, BaseConfig>;
pub type ConfigCollection = HashMap<String, ConfigSet>;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct BaseConfig {
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

impl Default for BaseConfig {
    /// By default the data is sqlite and
    /// the database is in memory
    fn default() -> Self {
        Self {
            enable: true,
            kind: DatabaseKind::Sqlite,
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

impl BaseConfig {
    pub fn kind(&self) -> DatabaseKind {
        self.kind.clone()
    }

    pub fn set_from(dirty_config: &DirtyConfig) -> ConfigSet {
        let config = dirty_config
            .optional_file("database.toml", Some("DTY_DB"))
            .build()
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
