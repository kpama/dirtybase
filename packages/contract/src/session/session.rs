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
        instance.storage.open(id).await;
        instance
    }

    pub async fn init(
        old_id: SessionId,
        storage: Arc<SessionStorageProvider>,
        lifetime: i64,
    ) -> Self {
        if let Some(data) = storage.get(&old_id).await {
            if !data.has_expired(lifetime) {
                tracing::event!(
                    tracing::Level::TRACE,
                    "Session {} is still valid",
                    old_id.to_string()
                );

                let session = Self {
                    id: old_id,
                    storage,
                };
                session.touch().await;
                return session;
            } else {
                storage.remove(&old_id).await;
            }
        }

        tracing::event!(
            tracing::Level::TRACE,
            "Session {} has expired. Generating a new one",
            old_id.to_string()
        );

        Self::new(SessionId::new(), storage).await
    }

    pub async fn get<T: DeserializeOwned>(&self, name: &str) -> Option<T> {
        if let Some(bucket) = self.storage.get(&self.id).await {
            if let Some(value) = bucket.get(name) {
                return serde_json::from_str(&value).ok();
            }
        }

        None
    }

    pub async fn put<V: serde::Serialize>(&self, name: &str, value: V) {
        if let Some(mut bucket) = self.storage.get(&self.id).await {
            bucket.add(
                name.to_string(),
                serde_json::to_string(&value).unwrap_or_default(),
            );
            self.storage.store(self.id.clone(), bucket).await;
        }
    }

    pub fn id(&self) -> SessionId {
        self.id.clone()
    }

    pub async fn touch(&self) {
        if let Some(mut bucket) = self.storage.get(&self.id).await {
            bucket.touch();
            self.storage.store(self.id.clone(), bucket).await;
        }
    }

    pub async fn has_expired(&self, lifetime: i64) -> bool {
        if let Some(bucket) = self.storage.get(&self.id).await {
            return bucket.has_expired(lifetime);
        }
        true
    }

    pub async fn delete(self) {
        _ = self.storage.remove(&self.id).await;
    }

    // TODO: Not sure we  need this method...
    pub async fn invalidate(self) -> Self {
        self.storage.remove(&self.id).await;
        Self::new(SessionId::new(), self.storage).await
    }
}
