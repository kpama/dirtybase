use std::collections::HashMap;

use dirtybase_helper::time::now_ts;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionData {
    created_at: i64,
    updated_at: i64,
    data: HashMap<String, String>,
}

impl Default for SessionData {
    fn default() -> Self {
        let ts = dirtybase_helper::time::now_ts();
        Self {
            created_at: ts,
            updated_at: ts,
            data: HashMap::default(),
        }
    }
}

impl SessionData {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn created_at(&self) -> i64 {
        self.created_at
    }

    pub fn touch(&mut self) {
        self.updated_at = now_ts()
    }

    pub fn updated_at(&self) -> i64 {
        self.updated_at
    }

    pub fn has_expired(&self, lifetime: i64) -> bool {
        self.updated_at + lifetime < now_ts()
    }

    pub fn add(&mut self, key: String, value: String) {
        self.data.insert(key, value);
        self.touch();
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    pub fn delete(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }

    pub fn has(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}
