#[derive(Debug, Clone)]
pub struct CacheConfig {
    cache_store: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_store: "db".to_string(),
        }
    }
}

impl CacheConfig {
    pub fn cache_store(&self) -> &String {
        &self.cache_store
    }
}

// #[busybody::async_trait]
// impl busybody::Injectable for CacheConfig {
//     async fn inject(_container: &busybody::ServiceContainer) -> Self {
//         Self::default()
//     }
// }
