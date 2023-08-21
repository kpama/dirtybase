use crate::app::core::time::now;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub value: String,
    pub expiration: Option<i64>,
}

impl CacheEntry {
    pub fn new(key: &str, value: &str, expiration: Option<i64>) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
            expiration,
        }
    }

    pub fn has_expired(&self) -> bool {
        if self.expiration.is_some() {
            return self.expiration.unwrap() < now().timestamp();
        }
        false
    }

    pub fn still_hot(&self) -> bool {
        !self.has_expired()
    }
}
