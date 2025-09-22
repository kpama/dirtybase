use std::sync::Arc;

use crate::lock_contract::{storage::LockStorage, LockData};

pub struct LockStorageProvider(Arc<Box<dyn LockStorage>>);

impl LockStorageProvider {
    pub fn new<T>(inner: T) -> Self
    where
        T: LockStorage + 'static,
    {
        Self(Arc::new(Box::new(inner)))
    }
}

#[async_trait::async_trait]
impl LockStorage for LockStorageProvider {
    async fn get(&self, key: &str) -> Result<Option<LockData>, anyhow::Error> {
        self.0.get(key).await
    }

    async fn set(&self, lock: LockData) -> Result<LockData, anyhow::Error> {
        self.0.set(lock).await
    }

    async fn restore(&self, key: &str, owner: &str) -> Result<Option<LockData>, anyhow::Error> {
        self.0.restore(key, owner).await
    }

    async fn delete(&self, lock: LockData) -> Result<(), anyhow::Error> {
        self.0.delete(lock).await
    }
}
