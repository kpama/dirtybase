mod dummy_storage;

use std::{fmt::Display, sync::Arc};

use busybody::async_trait;
use dirtybase_helper::uuid::Uuid25;
use dummy_storage::DummyStorage;

use super::SessionData;

#[derive(Debug, Hash, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionId(Arc<Uuid25>);

impl SessionId {
    pub fn new() -> Self {
        Self(Arc::new(dirtybase_helper::uuid::uuid25_v7()))
    }

    pub fn from_str(input: &str) -> Option<Self> {
        if let Some(id) = dirtybase_helper::uuid::uuid25_from_str(&input) {
            Some(Self(Arc::new(id)))
        } else {
            None
        }
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
    /// Start a session for the specified ID
    async fn open(&self, id: SessionId);

    /// Persists the provided data
    async fn store(&self, id: SessionId, value: SessionData);

    /// Retrieves the data with the specified key
    async fn get(&self, id: &SessionId) -> Option<SessionData>;

    /// Deletes and return the data with with specify key
    async fn remove(&self, id: &SessionId) -> Option<SessionData>;

    async fn gc(&self, lifetime: i64);
}

pub type SessionStorageProviderService = Arc<SessionStorageProvider>;

pub struct SessionStorageProvider(Box<dyn SessionStorage>);

#[async_trait]
impl SessionStorage for SessionStorageProvider {
    async fn open(&self, id: SessionId) {
        self.0.open(id).await;
    }

    async fn store(&self, id: SessionId, value: SessionData) {
        self.0.store(id, value).await;
    }

    async fn get(&self, id: &SessionId) -> Option<SessionData> {
        self.0.get(id).await
    }

    async fn remove(&self, id: &SessionId) -> Option<SessionData> {
        self.0.remove(id).await
    }

    async fn gc(&self, lifetime: i64) {
        self.0.gc(lifetime).await;
    }
}

impl Default for SessionStorageProvider {
    fn default() -> Self {
        Self(Box::new(DummyStorage::default()))
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
