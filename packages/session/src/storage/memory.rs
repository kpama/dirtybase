use std::{collections::HashMap, sync::Arc};

use busybody::async_trait;
use dirtybase_contract::session_contract::{
    SessionData, SessionId, SessionStorage, SessionStorageProvider,
};
use tokio::sync::RwLock;

use crate::SessionStorageResolver;

pub const NAME: &str = "memory";

#[derive(Debug, Default, Clone)]
pub struct MemoryStorage {
    storage: Arc<RwLock<HashMap<SessionId, SessionData>>>,
}

impl MemoryStorage {
    pub async fn register() {
        SessionStorageResolver::register(NAME, resolver).await;
    }
}

#[async_trait]
impl SessionStorage for MemoryStorage {
    async fn store(&self, id: SessionId, value: SessionData) {
        let mut w_lock = self.storage.write().await;
        w_lock.insert(id, value);
    }

    async fn get(&self, id: &SessionId) -> SessionData {
        let r_lock = self.storage.read().await;
        if let Some(data) = r_lock.get(id).cloned() {
            return data;
        }
        drop(r_lock);

        self.store(id.clone(), SessionData::new()).await;
        self.get(id).await
    }

    async fn remove(&self, id: &SessionId) -> Option<SessionData> {
        let mut w_lock = self.storage.write().await;
        w_lock.remove(id)
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

pub async fn resolver(
    resolver: SessionStorageResolver,
) -> Result<SessionStorageProvider, anyhow::Error> {
    let storage = MemoryStorage::default();
    let storage2 = storage.clone();

    // FIXME: This should cover all storage types
    let lifetime = resolver.config_ref().lifetime();
    let id = "session::storage".try_into().unwrap();
    let _ctx = dirtybase_cron::CronJob::schedule(
        "every 5 minutes",
        move |_| {
            Box::pin({
                let storage = storage2.clone();
                async move {
                    storage.gc(lifetime).await;
                }
            })
        },
        id,
    )
    .await;

    Ok(SessionStorageProvider::new(storage))
}
