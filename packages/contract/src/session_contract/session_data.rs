use std::{
    collections::HashMap,
    sync::{atomic::AtomicI64, Arc, RwLock},
};

use dirtybase_helper::time::now_ts;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionData {
    created_at: Arc<AtomicI64>,
    updated_at: Arc<AtomicI64>,
    inner: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for SessionData {
    fn default() -> Self {
        let ts = dirtybase_helper::time::now_ts();
        Self {
            created_at: Arc::new(ts.into()),
            updated_at: Arc::new(ts.into()),
            inner: Arc::default(),
        }
    }
}

impl SessionData {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_from(data: HashMap<String, String>, created_at: i64, updated_at: i64) -> Self {
        Self {
            inner: Arc::new(RwLock::new(data)),
            created_at: Arc::new(created_at.into()),
            updated_at: Arc::new(updated_at.into()),
        }
    }

    pub fn created_at(&self) -> i64 {
        self.created_at.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn touch(&self) {
        self.updated_at
            .fetch_add(now_ts(), std::sync::atomic::Ordering::Relaxed);
    }

    pub fn updated_at(&self) -> i64 {
        self.updated_at.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn has_expired(&self, lifetime: i64) -> bool {
        self.updated_at() + lifetime < now_ts()
    }

    pub fn add<K: ToString, V: ToString>(&self, key: K, value: V) {
        if let Ok(mut w_lock) = self.inner.write() {
            w_lock.insert(key.to_string(), value.to_string());
        }
        self.touch();
    }

    pub fn get(&self, key: &str) -> Option<String> {
        if let Ok(r_lock) = self.inner.read() {
            return r_lock.get(key).cloned();
        }
        None
    }

    pub fn delete(&self, key: &str) -> Option<String> {
        match self.inner.write() {
            Ok(mut w_lock) => w_lock.remove(key),
            Err(e) => {
                tracing::error!("could not delete session data: {}", e);
                None
            }
        }
    }

    pub fn has(&self, key: &str) -> bool {
        match self.inner.read() {
            Ok(r_lock) => r_lock.contains_key(key),
            Err(_) => false,
        }
    }

    pub fn all(&self) -> HashMap<String, String> {
        match self.inner.read() {
            Ok(r_lock) => r_lock.clone(),
            Err(_) => HashMap::new(),
        }
    }

    pub fn reset(&self) {
        if let Ok(mut w_lock) = self.inner.write() {
            _ = w_lock.drain();
        }
    }
}
