use dirtybase_contract::{
    app_contract::Context,
    config_contract::{ConfigResult, DirtyConfig, TryFromDirtyConfig},
};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CacheConfig {
    storage: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            storage: String::from("memory"),
        }
    }
}

impl CacheConfig {
    pub fn storage(&self) -> String {
        self.storage.clone()
    }

    pub fn storage_ref(&self) -> &str {
        self.storage.as_str()
    }
}

#[async_trait::async_trait]
impl TryFromDirtyConfig for CacheConfig {
    type Returns = Self;
    async fn from_config(config: &DirtyConfig, _ctx: &Context) -> ConfigResult<Self::Returns> {
        let con: Self = config
            .optional_file("cache.toml", Some("DTY_CACHE"))
            .build()
            .await?
            .try_deserialize()
            .unwrap_or_default();

        Ok(con)
    }
}
