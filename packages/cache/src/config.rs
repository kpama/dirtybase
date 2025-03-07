use dirtybase_contract::config::{ConfigResult, DirtyConfig, TryFromDirtyConfig};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CacheConfig {
    cache_store: Option<String>,
}

impl CacheConfig {
    pub async fn new(config: &DirtyConfig) -> Self {
        Self::from_config(config).await.unwrap()
    }

    pub fn cache_store(&self) -> &String {
        self.cache_store.as_ref().unwrap()
    }
}

#[async_trait::async_trait]
impl TryFromDirtyConfig for CacheConfig {
    type Returns = Self;
    async fn from_config(config: &DirtyConfig) -> ConfigResult<Self::Returns> {
        let mut con: Self = config
            .optional_file("cache.toml", Some("DTY_CACHE"))
            .build()
            .await?
            .try_deserialize()?;

        if con.cache_store.is_none() {
            con.cache_store = Some("memory".to_string());
        }

        Ok(con)
    }
}
