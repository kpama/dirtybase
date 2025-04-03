use std::sync::Arc;

use serde::de::DeserializeOwned;

use super::{SessionId, SessionStorage, SessionStorageProvider};

#[derive(Clone)]
pub struct Session {
    id: SessionId,
    storage: Arc<SessionStorageProvider>,
}

impl Session {
    pub(crate) async fn new(id: SessionId, storage: Arc<SessionStorageProvider>) -> Self {
        let instance = Self {
            id: id.clone(),
            storage,
        };
        instance
    }

    pub async fn init(
        id: SessionId,
        storage: Arc<SessionStorageProvider>,
        lifetime: i64,
        fingerprint: &str,
    ) -> Self {
        let mut session = Self::new(id, storage).await;
        let is_valid = if fingerprint.is_empty() || fingerprint != session.fingerprint().await {
            false
        } else {
            if session.has_expired(lifetime).await {
                false
            } else {
                true
            }
        };

        if is_valid {
            session.touch().await;
        } else {
            session = session.invalidate().await;
            session.set_fingerprint(fingerprint).await;
        }

        session
    }

    pub async fn get<T: DeserializeOwned>(&self, name: &str) -> Option<T> {
        let bucket = self.storage.get(&self.id).await;
        if let Some(value) = bucket.get(name) {
            return serde_json::from_str(&value).ok();
        }

        None
    }

    pub async fn put<V: serde::Serialize>(&self, name: &str, value: V) {
        let bucket = self.storage.get(&self.id).await;
        bucket.add(
            name.to_string(),
            serde_json::to_string(&value).unwrap_or_default(),
        );
        self.storage.store(self.id.clone(), bucket).await;
    }

    pub fn id(&self) -> SessionId {
        self.id.clone()
    }

    pub async fn touch(&self) {
        let bucket = self.storage.get(&self.id).await;
        bucket.touch();
        self.storage.store(self.id.clone(), bucket).await;
    }

    pub async fn has_expired(&self, lifetime: i64) -> bool {
        let bucket = self.storage.get(&self.id).await;
        bucket.has_expired(lifetime)
    }

    pub async fn delete(self) {
        _ = self.storage.remove(&self.id).await;
    }

    pub async fn invalidate(self) -> Self {
        self.storage.remove(&self.id).await;
        Self::new(SessionId::new(), self.storage).await
    }

    async fn fingerprint(&self) -> String {
        self.get("_fp").await.unwrap_or_default()
    }

    async fn set_fingerprint(&self, value: &str) {
        self.put("_fp", value).await
    }
}
