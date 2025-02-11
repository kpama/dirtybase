use std::{
    collections::HashMap,
    sync::{atomic::AtomicI64, Arc},
};

use futures::future::BoxFuture;
use tokio::{sync::RwLock, time::sleep};

type ContextCollection<T> = Arc<RwLock<HashMap<String, ContextWrapper<T>>>>;

struct ContextWrapper<T: Clone + Sync + Sync + 'static> {
    context: T,
    last_ts: AtomicI64,
    idle_duration: i64, // In seconds
}

impl<T: Clone + Sync + Sync + 'static> ContextWrapper<T> {
    pub fn new(context: T, idle_duration: i64) -> Self {
        Self {
            context: context,
            last_ts: AtomicI64::new(std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() as i64),
            idle_duration,
        }
    }

    fn context(&self) -> T {
        self.last_ts.swap(
            std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() as i64,
            std::sync::atomic::Ordering::Relaxed,
        );
        self.context.clone()
    }

    fn is_expired(&self) -> bool {
        let current_ts = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() as i64;

        current_ts - self.last_ts.load(std::sync::atomic::Ordering::Relaxed) > self.idle_duration
            && self.idle_duration > 0
    }
}

pub struct ContextManager<T: Clone + Send + Sync + 'static> {
    collection: ContextCollection<T>,
}

impl<T: Clone + Send + Sync + 'static> Default for ContextManager<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + Sync + 'static> ContextManager<T> {
    pub fn new() -> Self {
        Self {
            collection: ContextCollection::default(),
        }
    }

    pub async fn context<F>(
        &self,
        name: &str,
        idle_duration: i64, // in seconds
        mut else_callback: F,
    ) -> T
    where
        F: FnMut() -> BoxFuture<'static, T>,
    {
        let mut lock = self.collection.write().await;

        if !lock.contains_key(name) {
            lock.insert(
                name.to_string(),
                ContextWrapper::new(else_callback().await, idle_duration),
            );

            if idle_duration > 0 {
                let list = Arc::clone(&self.collection);
                let name2 = name.to_string();

                tokio::spawn(async move {
                    loop {
                        sleep(std::time::Duration::from_secs(2)).await;
                        let read_lock = list.read().await;

                        if read_lock.contains_key(&name2) {
                            if read_lock.get(&name2).unwrap().is_expired() {
                                drop(read_lock);
                                let mut write_lock = list.write().await;
                                write_lock.remove(&name2);
                                drop(write_lock);
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                });
            }
        }

        lock.get(name).unwrap().context()
    }

    pub async fn has_context(&self, name: &str) -> bool {
        let lock = self.collection.read().await;
        lock.contains_key(name)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_context_creation() {
        let manager = ContextManager::new();

        let counter = manager
            .context("counter", 1, || Box::pin(async { 100 }))
            .await;

        assert_eq!(counter, 100);
    }

    #[tokio::test]
    async fn test_context_creation2() {
        #[derive(Debug, Clone)]
        struct DbConnection {
            url: String,
        }
        let manager = ContextManager::new();
        let url_string = "connection_sting";
        let connection = manager
            .context("db_connection", 1, || {
                Box::pin({
                    async move {
                        let url = url_string.to_string();
                        DbConnection { url }
                    }
                })
            })
            .await;

        assert_eq!(&connection.url, url_string);
    }

    #[tokio::test]
    async fn test_idle_time_expired() {
        let manager = ContextManager::new();

        _ = manager
            .context("counter", 3, || Box::pin(async { 100 }))
            .await;

        sleep(std::time::Duration::from_secs(6)).await;
        assert!(!(manager.has_context("counter").await));
    }

    #[tokio::test]
    async fn test_idle_time_not_expired() {
        let manager = ContextManager::new();

        _ = manager
            .context("counter", 3, || Box::pin(async { 100 }))
            .await;

        assert!(manager.has_context("counter").await);
    }
}
