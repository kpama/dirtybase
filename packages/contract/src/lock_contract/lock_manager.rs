use std::time::Duration;

use crate::lock_contract::{
    storage::{LockStorage, LockStorageProvider},
    Lock, LockData,
};

#[derive(Clone)]
pub struct LockManager {
    storage: LockStorageProvider,
}

pub(crate) enum LockCommand {
    Acquire((tokio::sync::mpsc::Sender<bool>, LockData)),
    Release(LockData),
    Expires(LockData),
    Hibernate(LockData),
}

impl LockManager {
    pub fn new(storage: LockStorageProvider) -> Self {
        Self { storage }
    }

    /// Makes a lock using default values.
    ///
    /// TTL is set to 300 seconds by default
    pub fn make_default(&self, key: &str) -> Lock {
        let mut data = LockData::default();
        data.set_key(key);
        self.init_lock(data, true)
    }

    /// Make a new lock
    ///
    pub fn make(&self, key: &str, expires: i64) -> Lock {
        let data = LockData::new(key, expires);
        self.init_lock(data, true)
    }

    /// Tries to restore a lock that was acquired but not released
    ///
    /// The lock will be release when dropped. Call `resumable` if you want to
    /// prevent this behavior
    pub async fn restore(&self, owner: &str) -> Result<Lock, anyhow::Error> {
        let pieces = owner.split("||").map(String::from).collect::<Vec<String>>();

        if pieces.len() != 2 {
            return Err(anyhow::anyhow!("owner value is now in the right format"));
        }

        if let Ok(Some(data)) = self.storage.get(&pieces[0]).await {
            if data.owner().as_str() == owner {
                return Ok(self.init_lock(data, true));
            }
        }

        Err(anyhow::anyhow!(
            "lock with owner '{}' does not exist",
            owner
        ))
    }

    fn init_lock(&self, data: LockData, release_on_drop: bool) -> Lock {
        let manager = self.clone();
        let data2 = data.clone();
        let (tx, mut rx) = tokio::sync::mpsc::channel::<LockCommand>(4);
        tokio::spawn(async move {
            loop {
                while let Some(cmd) = rx.recv().await {
                    match cmd {
                        LockCommand::Acquire((sender, data)) => {
                            if let Ok(Some(d)) = manager.storage.get(data.key().as_str()).await {
                                if d.is_acquired() {
                                    _ = sender.send(false).await;
                                    continue;
                                }

                                if d.owner() != data.owner() && !d.is_blocking() {
                                    _ = manager.storage.set(data).await;
                                    _ = sender.send(true).await;
                                    continue;
                                }

                                if d.owner() == data.owner() {
                                    _ = manager.storage.set(data).await;
                                    _ = sender.send(true).await;
                                    continue;
                                }
                            }

                            if let Ok(_) = manager.storage.set(data).await {
                                _ = sender.send(true).await;
                                continue;
                            }

                            _ = sender.send(false).await;
                        }

                        LockCommand::Expires(data) | LockCommand::Release(data) => {
                            if let Err(e) = manager.storage.delete(data).await {
                                tracing::error!("could not delete global lock: {}", e);
                            }
                            rx.close();
                        }
                        LockCommand::Hibernate(data) => {
                            _ = manager.storage.set(data).await;
                            rx.close();
                        }
                    }
                }
            }
        });

        let tx2 = tx.clone();
        let expires = data.expires();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(expires as u64)).await;
            _ = tx2.clone().send(LockCommand::Expires(data2)).await;
        });

        Lock::new(data, tx, release_on_drop)
    }
}

#[cfg(test)]
mod test {
    use crate::lock_contract::storage::LockMemoryStorage;

    use super::*;

    #[tokio::test]
    async fn test_lock_acquiring() {
        let manager = LockManager::new(LockStorageProvider::new(LockMemoryStorage::new().await));
        let mut lock = manager.make("test-lck-1", 2);
        assert!(lock.acquire(1).await, "could not successfully acquire lock");

        let mut lock2 = manager.make("test-lck-1", 2);
        assert!(
            !lock2.acquire(1).await,
            "should not successfully acquire lock"
        );

        drop(lock);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let mut lock3 = manager.make("test-lck-1", 2);
        assert!(lock3.acquire(1).await, "should have acquired lock");
    }

    #[tokio::test]
    async fn test_dropping() {
        let manager = LockManager::new(LockStorageProvider::new(LockMemoryStorage::new().await));
        let mut lock = manager.make("test-lck-dropping", 1);
        assert!(lock.acquire(1).await, "could not successfully acquire lock");
        drop(lock);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let mut lock2 = manager.make("test-lck-dropping", 1);
        assert!(
            lock2.acquire(1).await,
            "could not successfully acquire lock"
        );
    }

    #[tokio::test]
    async fn test_lock_restoring() {
        let manager = LockManager::new(LockStorageProvider::new(LockMemoryStorage::new().await));
        let mut lock = manager.make("test-lck-restore1", 2);

        lock.store().await;

        let lock2 = manager
            .restore(&lock.owner())
            .await
            .expect("could not fetch lock from owner value");

        assert_eq!(lock.owner(), lock2.owner(), "restored lock is different");
    }

    #[tokio::test]
    async fn test_sharing_lock() {
        let manager = LockManager::new(LockStorageProvider::new(LockMemoryStorage::new().await));

        let mut lock = manager.make("test-lck-share1", 2);
        assert!(lock.acquire_shared(1).await);

        let mut lock2 = manager.make("test-lck-share1", 2);
        assert!(lock2.acquire(1).await);

        let mut lock3 = manager.make("test-lck-share1", 2);
        assert!(
            !lock3.acquire_shared(1).await,
            "lock was acquire exclusively by lock2"
        );

        assert!(
            lock2.make_share().await,
            "lock2 should be able to make the lock shareable"
        );

        let mut lock4 = manager.make("test-lck-share1", 2);
        assert!(
            lock4.acquire_shared(1).await,
            "lock4 should have gotten shared access"
        );
    }
}
