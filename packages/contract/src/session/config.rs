use std::sync::Arc;

use crate::config::DirtyConfig;

use super::DEFAULT_LIFETIME;

#[derive(Debug, Clone, serde::Deserialize)]
pub enum SessionStorageDriver {
    #[serde(alias = "custom")]
    Custom(String),
    #[serde(alias = "dummy")]
    Dummy,
    #[serde(alias = "database")]
    Database,
    #[serde(alias = "file")]
    File,
    #[serde(alias = "memory")]
    Memory,
    #[serde(alias = "redis")]
    Redis,
}

#[derive(Debug, serde::Deserialize)]
pub struct SessionConfig {
    driver: SessionStorageDriver,
    lifetime: i64,
    cookie: Arc<String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            driver: SessionStorageDriver::Dummy,
            lifetime: DEFAULT_LIFETIME as i64 * 60,
            cookie: "dty_session".to_string().into(),
        }
    }
}

impl SessionConfig {
    pub fn driver(&self) -> SessionStorageDriver {
        self.driver.clone()
    }

    pub fn driver_ref(&self) -> &SessionStorageDriver {
        &self.driver
    }

    pub fn lifetime(&self) -> i64 {
        self.lifetime
    }

    pub fn cookie_ref(&self) -> &str {
        self.cookie.as_ref()
    }

    pub fn cookie(&self) -> Arc<String> {
        self.cookie.clone()
    }
}

impl From<&DirtyConfig> for SessionConfig {
    fn from(base: &DirtyConfig) -> Self {
        base.optional_file("session.toml", Some("DTY_SESSION"))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }
}
