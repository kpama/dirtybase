use dirtybase_contract::{
    app_contract::Context,
    config_contract::{ConfigResult, DirtyConfig, TryFromDirtyConfig},
};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CacheConfig {
    cache_store: Option<String>,
}

impl CacheConfig {
    pub fn cache_store(&self) -> &String {
        self.cache_store.as_ref().unwrap()
    }
}

#[async_trait::async_trait]
impl TryFromDirtyConfig for CacheConfig {
    type Returns = Self;
    async fn from_config(config: &DirtyConfig, _ctx: &Context) -> ConfigResult<Self::Returns> {
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
