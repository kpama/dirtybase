use crate::lock_contract::LockData;

#[async_trait::async_trait]
pub trait LockStorage: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<LockData>, anyhow::Error>;
    async fn restore(&self, key: &str, owner: &str) -> Result<Option<LockData>, anyhow::Error>;
    async fn set(&self, lock: LockData) -> Result<LockData, anyhow::Error>;
    async fn delete(&self, lock: LockData) -> Result<(), anyhow::Error>;
}
