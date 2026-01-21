use crate::lock_contract::LockData;

#[async_trait::async_trait]
pub trait LockStorage: Send + Sync {
    /// Get an existing lock data
    async fn get(&self, key: &str) -> Result<Option<LockData>, anyhow::Error>;

    /// Update an existing lock data
    async fn set(&self, lock: LockData) -> Result<LockData, anyhow::Error>;

    /// Delete a lock data
    async fn delete(&self, lock: LockData) -> Result<(), anyhow::Error>;
}
