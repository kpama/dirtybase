use anyhow::anyhow;
use dirtybase_contract::{
    config::{ConfigResult, DirtyConfig, TryFromDirtyConfig},
    session::DEFAULT_LIFETIME,
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SessionConfig {
    storage: SessionStorageDriver,
    lifetime: i64,
    #[serde(default = "default_session_id")]
    cookie_id: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            storage: SessionStorageDriver::Dummy,
            lifetime: DEFAULT_LIFETIME as i64 * 60,
            cookie_id: "dty_session".to_string().into(),
        }
    }
}

#[async_trait::async_trait]
impl TryFromDirtyConfig for SessionConfig {
    type Returns = Self;
    async fn from_config(config: &DirtyConfig) -> ConfigResult<Self::Returns> {
        match config
            .optional_file("session.toml", Some("DTY_SESSION"))
            .build()
            .await?
            .try_deserialize()
        {
            Ok(c) => Ok(c),
            Err(e) => {
                let message = format!("could not build session config: {}", e.to_string());
                tracing::error!("{}", &message);
                Err(anyhow!(e))
            }
        }
    }
}

impl SessionConfig {
    pub fn storage(&self) -> SessionStorageDriver {
        self.storage.clone()
    }

    pub fn storage_ref(&self) -> &SessionStorageDriver {
        &self.storage
    }

    pub fn lifetime(&self) -> i64 {
        self.lifetime
    }

    pub fn cookie_id_ref(&self) -> &str {
        self.cookie_id.as_ref()
    }

    pub fn cookie_id(&self) -> String {
        self.cookie_id.clone()
    }

    pub async fn from_dirty_config(config: &DirtyConfig) -> Self {
        Self::from_config(config).await.unwrap()
    }
}

fn default_session_id() -> String {
    "dty_session".to_string()
}
