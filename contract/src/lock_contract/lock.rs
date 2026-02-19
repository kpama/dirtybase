use tokio::sync::mpsc::error::SendError;

use crate::{
    db_contract::types::StringField,
    lock_contract::{LockCommand, LockData},
};

pub struct Lock {
    data: LockData,
    drop: bool,
    acquired: bool,
    tx: tokio::sync::mpsc::Sender<LockCommand>,
}

impl Lock {
    pub(super) fn new(
        data: LockData,
        tx: tokio::sync::mpsc::Sender<LockCommand>,
        drop: bool,
    ) -> Self {
        Self {
            data,
            tx,
            drop,
            acquired: false,
        }
    }

    /// Returns the lock owner's value
    ///
    /// If you are planning to pass this value between processes,
    /// make sure to call the `store` method at some point
    pub fn owner(&self) -> StringField {
        self.data.owner()
    }

    /// Stores the current lock for future restoring
    ///
    /// This causes the lock to not be dropped at the end of the scope
    pub async fn store(&mut self) -> bool {
        if !self.acquired && !self.acquire(5).await {
            return false;
        }
        self.drop = false;
        _ = Self::send_command(self.tx.clone(), LockCommand::Hibernate(self.data.clone())).await;
        self.acquired = true;

        self.acquired
    }

    /// Tries to acquire the lock and make it shareable
    ///
    /// Providing the owner's value will give you access to the lock
    pub async fn acquire_shared(&mut self, wait_for: i64) -> bool {
        self.data.blocking = false;
        let data = self.data.clone();
        self.acquired = Self::do_acquiring(self.tx.clone(), data, wait_for).await;
        self.acquired
    }

    /// Makes an exclusive lock shareable
    ///
    /// This method only works when you already have acquire exclusive access to the lock
    pub async fn make_share(mut self) -> bool {
        if !self.acquired || !self.data.blocking {
            return false;
        }

        self.data.acquired = false;
        self.data.blocking = false;
        self.drop = false;
        self.acquired = false;
        Self::send_command(self.tx.clone(), LockCommand::Hibernate(self.data.clone()))
            .await
            .is_ok()
    }

    /// Tries to acquire the lock
    ///
    /// The lock is tie to one user. If you want to share a lock use the
    /// `acquire_shared` lock
    pub async fn acquire(&mut self, wait_for: i64) -> bool {
        self.data.acquired = true;
        let data = self.data.clone();
        self.acquired = Self::do_acquiring(self.tx.clone(), data, wait_for).await;
        self.acquired
    }

    async fn do_acquiring(
        manager_tx: tokio::sync::mpsc::Sender<LockCommand>,
        data: LockData,
        wait_for: i64,
    ) -> bool {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<bool>(1);
        let tx2 = tx.clone();

        // timer
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(wait_for as u64)).await;
            _ = tx2.send(false).await;
        });

        _ = Self::send_command(manager_tx, LockCommand::Acquire((tx, data))).await;

        if let Some(r) = rx.recv().await {
            return r;
        }

        return false;
    }

    async fn send_command(
        manager_tx: tokio::sync::mpsc::Sender<LockCommand>,
        cmd: LockCommand,
    ) -> Result<(), SendError<LockCommand>> {
        manager_tx.send(cmd).await
    }

    /// Releases the lock
    ///
    /// If the lock is a shareable lock, use the force_release
    pub async fn release(mut self) {
        if !self.acquired {
            return;
        }

        if self.data.blocking {
            self.force_release().await;
        } else {
            self.drop = false;
            let mut data = self.data.clone();
            data.acquired = false;
            _ = self.tx.send(LockCommand::Hibernate(data));
        }
    }

    /// Releases the lock disregarding any existing condition
    pub async fn force_release(mut self) {
        if !self.acquired {
            return;
        }

        self.drop = false;
        _ = self.tx.send(LockCommand::Release(self.data.clone()));
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        if !self.acquired {
            return;
        }

        let tx = self.tx.clone();
        let data = self.data.clone();
        let drop = self.drop;
        tokio::spawn(async move {
            _ = tx
                .send(if drop && data.blocking {
                    LockCommand::Release(data)
                } else {
                    LockCommand::Hibernate(data)
                })
                .await;
        });
    }
}
