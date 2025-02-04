use std::{collections::HashMap, sync::RwLock};

use serde::Serialize;

#[derive(Default)]
pub struct ContextMetadata {
    data: RwLock<HashMap<String, String>>,
    hidden: RwLock<HashMap<String, String>>,
    stack: RwLock<Vec<String>>,
}

impl ContextMetadata {
    pub fn add(&self, key: &str, value: String) {
        if let Ok(mut w_lock) = self.data.write() {
            w_lock.insert(key.to_string(), value);
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        if let Ok(r_lock) = self.data.read() {
            r_lock.get(key).cloned()
        } else {
            None
        }
    }

    pub fn all(&self) -> HashMap<String, String> {
        if let Ok(r_lock) = self.data.read() {
            r_lock
                .iter()
                .map(|entry| (entry.0.clone(), entry.1.clone()))
                .collect::<HashMap<String, String>>()
        } else {
            HashMap::new()
        }
    }
}
