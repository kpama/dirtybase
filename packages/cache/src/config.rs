#[derive(Debug, Clone, serde::Deserialize)]
pub struct CacheConfig {
    cache_store: Option<String>,
}

impl CacheConfig {
    pub fn new(config: &dirtybase_config::DirtyConfig) -> Self {
        let mut con: Self = config
            .optional_file("cache.toml", Some("DTY_CACHE"))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();

        if con.cache_store.is_none() {
            con.cache_store = Some("memory".to_string());
        }

        con
    }

    pub fn cache_store(&self) -> &String {
        self.cache_store.as_ref().unwrap()
    }
}

// #[busybody::async_trait]
// impl busybody::Injectable for CacheConfig {
//     async fn inject(_container: &busybody::ServiceContainer) -> Self {
//         Self::default()
//     }
// }
