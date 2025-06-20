use dirtybase_db_macro::DirtyTable;
use dirtybase_helper::time::now;

type CacheContent = serde_json::Value;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, DirtyTable)]
#[dirty(id = "key")]
pub struct CacheEntry {
    pub key: String,
    pub value: CacheContent,
    pub expiration: Option<i64>,
}

impl CacheEntry {
    pub fn new(key: String, value: serde_json::Value, expiration: Option<i64>) -> Self {
        Self {
            value,
            key: key.to_string(),
            expiration,
        }
    }

    pub fn has_expired(&self) -> bool {
        if self.expiration.is_some() && self.expiration.unwrap() > 0 {
            return self.expiration.unwrap() < now().timestamp();
        }
        false
    }

    pub fn still_hot(&self) -> bool {
        !self.has_expired()
    }
}
