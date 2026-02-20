use std::sync::{Arc, RwLock, atomic::AtomicI64};

use dirtybase_helper::time::now_ts;
use serde::Serialize;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionData {
    expires: Arc<AtomicI64>,
    inner: Arc<RwLock<serde_json::Map<String, serde_json::Value>>>,
}

impl Default for SessionData {
    fn default() -> Self {
        let ts = dirtybase_helper::time::now_ts();
        Self {
            expires: Arc::new(ts.into()),
            inner: Arc::default(),
        }
    }
}

impl SessionData {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_from(data: serde_json::Map<String, serde_json::Value>, expires: i64) -> Self {
        Self {
            inner: Arc::new(RwLock::new(data)),
            expires: Arc::new(expires.into()),
        }
    }

    pub fn touch(&self, lifetime: i64) {
        let mut now = dirtybase_helper::time::Time::now();
        now = now.add_minutes(lifetime);
        self.expires
            .swap(now.timestamp(), std::sync::atomic::Ordering::Relaxed);
    }

    pub fn expires(&self) -> i64 {
        self.expires.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn has_expired(&self, lifetime: i64) -> bool {
        self.expires() + lifetime < now_ts()
    }

    pub fn add<K: ToString, V: Serialize>(&self, key: K, value: V) {
        if let Ok(mut w_lock) = self.inner.write() {
            w_lock.insert(
                key.to_string(),
                serde_json::to_value(value).unwrap_or_default(),
            );
        }
    }

    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        if let Ok(r_lock) = self.inner.read() {
            return r_lock.get(key).cloned();
        }
        None
    }

    pub fn delete(&self, key: &str) -> Option<serde_json::Value> {
        match self.inner.write() {
            Ok(mut w_lock) => w_lock.remove(key),
            Err(e) => {
                tracing::error!("could not delete session data: {}", e);
                None
            }
        }
    }

    pub fn has(&self, key: &str) -> bool {
        if let Ok(r_lock) = self.inner.read() {
            return r_lock.contains_key(key);
        }
        false
    }

    pub fn all(&self) -> serde_json::Map<String, serde_json::Value> {
        if let Ok(r_lock) = self.inner.read() {
            return r_lock.clone();
        }
        serde_json::Map::default()
    }

    pub fn reset(&self) {
        if let Ok(mut w_lock) = self.inner.write() {
            w_lock.clear();
        }
    }
}
