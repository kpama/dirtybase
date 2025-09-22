use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use dirtybase_contract::lock_contract::{LockData, storage::LockStorage};
use tokio::sync::RwLock;

type Storage = Arc<RwLock<HashMap<String, LockData>>>;

static LOCK_MEMORY_STORAGE: OnceLock<Storage> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct LockMemoryStorage(Storage);

impl LockMemoryStorage {
    pub async fn new() -> Self {
        Self(
            LOCK_MEMORY_STORAGE
                .get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
                .clone(),
        )
    }
}

#[async_trait::async_trait]
impl LockStorage for LockMemoryStorage {
    async fn get(&self, key: &str) -> Result<Option<LockData>, anyhow::Error> {
        let r_lock = self.0.read().await;
        Ok(r_lock.get(key).cloned())
    }
    async fn restore(&self, key: &str, owner: &str) -> Result<Option<LockData>, anyhow::Error> {
        // TODO: DO RESTORING
        self.get(key).await
    }
    async fn set(&self, lock: LockData) -> Result<LockData, anyhow::Error> {
        let mut w_lock = self.0.write().await;
        w_lock.insert(lock.key().to_string(), lock.clone());
        Ok(lock)
    }
    async fn delete(&self, lock: LockData) -> Result<(), anyhow::Error> {
        let existing = if let Ok(Some(existing)) = self.get(lock.key()).await {
            existing
        } else {
            return Ok(());
        };

        if existing.owner() != lock.owner() {
            return Err(anyhow::anyhow!("Cannot delete lock. Owner does not match"));
        }

        let mut w_lock = self.0.write().await;
        _ = w_lock.remove(lock.key());
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_set_mem_storage() {
        let storage = LockMemoryStorage::new().await;
        let lock_data = storage.set(LockData::new("test:lock-mem-set", 1)).await;
        assert!(lock_data.is_ok());
        assert_eq!(lock_data.unwrap().key(), "test:lock-mem-set");
    }

    #[tokio::test]
    async fn test_get_mem_storage() {
        let storage = LockMemoryStorage::new().await;
        let lock_data = storage.set(LockData::new("test:mem-get", 1)).await.unwrap();
        assert_eq!(lock_data.key(), "test:mem-get");
        let existing = storage.get("test:mem-get").await.unwrap().unwrap();
        assert_eq!(lock_data.key(), existing.key());
        assert_eq!(lock_data.expires(), existing.expires());
    }
}
