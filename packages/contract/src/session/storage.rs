use std::{fmt::Display, sync::Arc};

use busybody::async_trait;
use dirtybase_helper::security::random_bytes_hex;

use super::SessionData;

#[derive(Debug, Hash, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionId(Arc<String>);

impl SessionId {
    pub fn new() -> Self {
        Self(random_bytes_hex(32).into())
    }

    pub fn from_str(input: &str) -> Option<Self> {
        if let Ok(bytes) = hex::decode(input) {
            if bytes.len() == 32 {
                return Some(Self(input.to_string().into()));
            }
        }
        None
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&SessionId> for String {
    fn from(value: &SessionId) -> Self {
        value.0.to_string()
    }
}

#[async_trait]
pub trait SessionStorage: Send + Sync {
    /// Persists the provided data
    async fn store(&self, id: SessionId, value: SessionData);

    /// Retrieves the data with the specified key
    async fn get(&self, id: &SessionId) -> SessionData;

    /// Deletes and return the data with with specify key
    async fn remove(&self, id: &SessionId) -> Option<SessionData>;

    async fn gc(&self, lifetime: i64);
}

pub type SessionStorageProviderService = Arc<SessionStorageProvider>;

pub struct SessionStorageProvider(Box<dyn SessionStorage>);

#[async_trait]
impl SessionStorage for SessionStorageProvider {
    async fn store(&self, id: SessionId, value: SessionData) {
        self.0.store(id, value).await;
    }

    async fn get(&self, id: &SessionId) -> SessionData {
        self.0.get(id).await
    }

    async fn remove(&self, id: &SessionId) -> Option<SessionData> {
        self.0.remove(id).await
    }

    async fn gc(&self, lifetime: i64) {
        self.0.gc(lifetime).await;
    }
}

impl SessionStorageProvider {
    pub fn from<S>(storage: S) -> Self
    where
        S: SessionStorage + 'static,
    {
        Self(Box::new(storage))
    }
}
