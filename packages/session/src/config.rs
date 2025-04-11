use std::sync::Arc;

use anyhow::anyhow;
use dirtybase_contract::{
    app_contract::Context,
    config_contract::{ConfigResult, DirtyConfig, TryFromDirtyConfig},
    session_contract::DEFAULT_LIFETIME,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionConfig {
    storage: Arc<String>,
    lifetime: i64,
    #[serde(default = "default_session_id")]
    cookie_id: Arc<String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            storage: Arc::new("dummy".to_string()),
            lifetime: DEFAULT_LIFETIME as i64 * 60,
            cookie_id: "dty_session".to_string().into(),
        }
    }
}

#[async_trait::async_trait]
impl TryFromDirtyConfig for SessionConfig {
    type Returns = Self;
    async fn from_config(config: &DirtyConfig, _ctx: &Context) -> ConfigResult<Self::Returns> {
        match config
            .optional_file("session.toml", Some("DTY_SESSION"))
            .build()
            .await?
            .try_deserialize()
        {
            Ok(c) => Ok(c),
            Err(e) => {
                let message = format!("could not build session config: {}", e);
                tracing::error!("{}", &message);
                Err(anyhow!(e))
            }
        }
    }
}

impl SessionConfig {
    pub fn storage(&self) -> Arc<String> {
        self.storage.clone()
    }

    pub fn storage_ref(&self) -> &str {
        self.storage.as_str()
    }

    pub fn lifetime(&self) -> i64 {
        self.lifetime
    }

    pub fn cookie_id_ref(&self) -> &str {
        self.cookie_id.as_ref()
    }

    pub fn cookie_id(&self) -> Arc<String> {
        self.cookie_id.clone()
    }
}

fn default_session_id() -> Arc<String> {
    Arc::new("dty_session".to_string())
}
