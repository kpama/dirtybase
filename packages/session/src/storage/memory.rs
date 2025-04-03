use std::{collections::HashMap, sync::Arc};

use busybody::async_trait;
use dirtybase_contract::session::{
    SessionData, SessionId, SessionStorage, SessionStorageProvider, SessionStorageProviderService,
};
use tokio::sync::RwLock;

#[derive(Debug, Default, Clone)]
pub struct MemoryStorage {
    storage: Arc<RwLock<HashMap<SessionId, SessionData>>>,
}

#[async_trait]
impl SessionStorage for MemoryStorage {
    async fn store(&self, id: SessionId, value: SessionData) {
        let mut w_lock = self.storage.write().await;
        w_lock.insert(id, value);
    }

    async fn get(&self, id: &SessionId) -> SessionData {
        let r_lock = self.storage.read().await;
        if let Some(data) = r_lock.get(&id).cloned() {
            return data;
        }
        drop(r_lock);

        self.store(id.clone(), SessionData::new()).await;
        self.get(id).await
    }

    async fn remove(&self, id: &SessionId) -> Option<SessionData> {
        let mut w_lock = self.storage.write().await;
        w_lock.remove(&id)
    }

    async fn gc(&self, lifetime: i64) {
        let r_lock = self.storage.read().await;
        let mut to_remove = Vec::new();
        for (id, data) in r_lock.iter() {
            if data.has_expired(lifetime) {
                to_remove.push(id.clone());
            }
        }
        drop(r_lock);

        for id in to_remove.iter() {
            self.remove(id).await;
        }
    }
}

impl MemoryStorage {
    pub async fn make_provider() -> SessionStorageProviderService {
        let provider = busybody::helpers::service_container()
            .set_type(Arc::new(SessionStorageProvider::from(
                MemoryStorage::default(),
            )))
            .await
            .get_type::<SessionStorageProviderService>()
            .await
            .unwrap();

        provider
    }
}
