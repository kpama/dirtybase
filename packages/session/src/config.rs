use dirtybase_contract::{config::DirtyConfig, session::DEFAULT_LIFETIME};

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
    cookie: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            storage: SessionStorageDriver::Dummy,
            lifetime: DEFAULT_LIFETIME as i64 * 60,
            cookie: "dty_session".to_string().into(),
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

    pub fn cookie_ref(&self) -> &str {
        self.cookie.as_ref()
    }

    pub fn cookie(&self) -> String {
        self.cookie.clone()
    }
}

impl From<&DirtyConfig> for SessionConfig {
    fn from(base: &DirtyConfig) -> Self {
        match base
            .optional_file("session.toml", Some("DTY_SESSION"))
            .build()
            .expect("could not build dirtybase configuration for session config")
            .try_deserialize()
        {
            Ok(c) => c,
            Err(e) => {
                let message = format!("could not build session config: {}", e.to_string());
                tracing::error!("{}", &message);
                panic!("{}", message);
            }
        }
    }
}
