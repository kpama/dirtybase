use std::{
    collections::HashMap,
    sync::{atomic::AtomicI64, Arc},
};

use anyhow::anyhow;
use futures::future::BoxFuture;
use tokio::{sync::RwLock, time::sleep};

use super::Context;

type ContextCollection<T> = Arc<RwLock<HashMap<String, ResourceWrapper<T>>>>;

struct ResourceWrapper<T: Clone + Sync + Sync + 'static> {
    resource: T,
    last_ts: AtomicI64,
    idle_timeout: i64, // In seconds
}

impl<T: Clone + Sync + Sync + 'static> ResourceWrapper<T> {
    pub fn new(resource: T, idle_timeout: i64) -> Self {
        Self {
            resource,
            last_ts: AtomicI64::new(std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() as i64),
            idle_timeout,
        }
    }

    fn resource(&self) -> T {
        self.last_ts.swap(
            std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() as i64,
            std::sync::atomic::Ordering::Relaxed,
        );
        self.resource.clone()
    }

    fn is_expired(&self) -> bool {
        let current_ts = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() as i64;

        current_ts - self.last_ts.load(std::sync::atomic::Ordering::Relaxed) > self.idle_timeout
            && self.idle_timeout > 0
    }
}

#[derive(Clone)]
pub struct ContextResourceManager<T: Clone + Send + Sync + 'static> {
    setup_fn:
        Arc<RwLock<Box<dyn FnMut(Context) -> BoxFuture<'static, (String, i64)> + Send + Sync>>>,
    resolver_fn: Arc<
        RwLock<
            Box<dyn FnMut(Context) -> BoxFuture<'static, Result<T, anyhow::Error>> + Send + Sync>,
        >,
    >,
    drop_fn: Arc<RwLock<Box<dyn FnMut(T) -> BoxFuture<'static, ()> + Send + Sync>>>,
    collection: ContextCollection<T>,
}

impl<T: Clone + Send + Sync + 'static> ContextResourceManager<T> {
    /// Creates an instance of the ContextResourceManager
    ///
    /// This struct manages the life circle of a resource based on the
    /// current context.
    ///
    /// This method takes three closures
    ///  1. Setup: a closure that implements `FnMut(Context) -> BoxFuture<(String, i64)>`
    ///            The return type of this closure must be a tuple with two elements.
    ///            - Index 0: A string used to identify this specific instance of the resource.
    ///            - Index 1: where 0 means the resource will bre for the entire application life,
    ///                       less than 0 means the the fallback closure will be called each time an instance
    ///                       is required and value above zero means the resource will be dropped after being
    ///                       idle for this length of time.
    ///
    ///  2. Resolver: a closure the implements `FnMut(Context) -> BoxFuture<T>` where `T` is an
    ///               instance of the resource.
    ///
    ///  3. Drop: a closure that implements `FnMut(T) -> BoxFuture<()>` where T is the instance
    ///     that has been dropped ie. remove from he collection of instances
    ///
    pub async fn new<S, F, C>(setup_fn: S, resolver_fn: F, drop_fn: C) -> Self
    where
        S: FnMut(Context) -> BoxFuture<'static, (String, i64)> + Send + Sync + 'static,
        F: FnMut(Context) -> BoxFuture<'static, Result<T, anyhow::Error>> + Send + Sync + 'static,
        C: FnMut(T) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        let instance = Self {
            setup_fn: Arc::new(RwLock::new(Box::new(setup_fn))),
            resolver_fn: Arc::new(RwLock::new(Box::new(resolver_fn))),
            drop_fn: Arc::new(RwLock::new(Box::new(drop_fn))),
            collection: ContextCollection::default(),
        };

        // FIXME: look into a deadlock situation happening when we drop the instace of this struct
        // instance.handle_shutdown_signal().await

        instance
    }

    pub async fn register<S, F, C>(setup_fn: S, resolver_fn: F, drop_fn: C)
    where
        S: FnMut(Context) -> BoxFuture<'static, (String, i64)> + Send + Sync + 'static,
        F: FnMut(Context) -> BoxFuture<'static, Result<T, anyhow::Error>> + Send + Sync + 'static,
        C: FnMut(T) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        let instance = Self::new(setup_fn, resolver_fn, drop_fn).await;

        busybody::helpers::service_container().set(instance).await;
    }

    /// Register a resource that will only last for a scope request/command
    pub async fn scoped<S, R>(mut setup_fn: S, resolver_fn: R)
    where
        S: FnMut(Context) -> BoxFuture<'static, String> + Send + Sync + 'static,
        R: FnMut(Context) -> BoxFuture<'static, Result<T, anyhow::Error>> + Send + Sync + 'static,
    {
        Self::register(
            move |c| {
                let result = setup_fn(c);
                Box::pin(async {
                    let name = result.await;
                    (name, -1)
                })
            },
            resolver_fn,
            |_| {
                Box::pin(async {
                    // we never reach here
                })
            },
        )
        .await;
    }

    pub async fn try_get(context: &Context) -> Result<T, anyhow::Error> {
        if let Some(manager) = context.container().get::<Self>().await {
            let mut setup_fn_lock = manager.setup_fn.write().await;
            let (name, idle_timeout) = (setup_fn_lock)(context.clone()).await;

            return manager
                .get_resource(context.clone(), &name, idle_timeout)
                .await;
        }

        Err(anyhow!("resource not found"))
    }

    async fn get_resource(
        &self,
        context: Context,
        name: &str,
        idle_timeout: i64, // in seconds
    ) -> Result<T, anyhow::Error> {
        let mut lock = self.collection.write().await;

        if !lock.contains_key(name) {
            let mut resolver_lock = self.resolver_fn.write().await;
            let resource = (resolver_lock)(context).await;

            if resource.is_err() {
                return resource;
            }

            if idle_timeout > 0 {
                return resource;
            }

            lock.insert(
                name.to_string(),
                ResourceWrapper::new(resource.unwrap(), idle_timeout),
            );

            if idle_timeout > 0 {
                let list = Arc::clone(&self.collection);
                let name2 = name.to_string();
                let clean_up_fn = self.drop_fn.clone();

                tokio::spawn(async move {
                    loop {
                        sleep(std::time::Duration::from_secs(idle_timeout as u64)).await;
                        let read_lock = list.read().await;

                        if read_lock.contains_key(&name2) {
                            if read_lock.get(&name2).unwrap().is_expired() {
                                drop(read_lock);
                                let mut write_lock = list.write().await;
                                if let Some(ctx) = write_lock.remove(&name2) {
                                    let resource = ctx.resource();
                                    let mut clean_fn_lock = clean_up_fn.write().await;
                                    (clean_fn_lock)(resource).await;
                                    drop(ctx);
                                }

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

        Ok(lock.get(name).unwrap().resource())
    }

    pub async fn has_resource(&self, name: &str) -> bool {
        let lock = self.collection.read().await;
        lock.contains_key(name)
    }

    async fn drop_all(&self) {
        tracing::trace!(
            "shutting down resource context manager: {}",
            self.name_of_t()
        );
        let list = Arc::clone(&self.collection);
        let clean_up_fn = self.drop_fn.clone();
        let mut write_lock = list.write().await;
        for (_, wrapper) in write_lock.drain().into_iter() {
            let mut clean_fn_lock = clean_up_fn.write().await;
            (clean_fn_lock)(wrapper.resource()).await;
        }
    }

    fn name_of_t(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    async fn handle_shutdown_signal(self) -> Self {
        let this_manager = self.clone();
        // FIXME: use something else other than block_on
        //        block_on does not work for db over tcp connection

        tokio::spawn(async move {
            let ctrl_c = async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to install Ctrl+C handler");
            };

            #[cfg(unix)]
            let terminate = async {
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("failed to install signal handler")
                    .recv()
                    .await;
            };

            #[cfg(not(unix))]
            let terminate = std::future::pending::<()>();

            tokio::select! {
                _ = ctrl_c => {
                    tracing::error!("shutting down due to ctr+c");
                    // this_manager.drop_all().await;
                },
                _ = terminate => {
                    tracing::error!("shutting down for other reason");
                    // this_manager.drop_all().await;
                },
            }
        });

        self
    }
}

impl<T: Clone + Send + Sync + 'static> Drop for ContextResourceManager<T> {
    fn drop(&mut self) {
        // FIXME: use something else other than block_on
        //        block_on does not work for db over tcp connection

        // futures::executor::block_on(self.drop_all());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_context_creation() {
        let manager = ContextResourceManager::new(
            |_| Box::pin(async { ("global".to_owned(), 20) }),
            |_| Box::pin(async { Ok(100) }),
            |_| Box::pin(async {}),
        )
        .await;

        let counter = manager.get_resource(Context::default(), "counter", 1).await;

        assert_eq!(counter.unwrap(), 100);
    }

    #[tokio::test]
    async fn test_context_creation2() {
        #[derive(Debug, Clone)]
        struct DbConnection {
            url: String,
        }

        let url_string = "connection_sting";
        let manager = ContextResourceManager::new(
            |_| Box::pin(async { ("global".to_owned(), 20) }),
            move |_| {
                Box::pin({
                    async move {
                        let url = url_string.to_string();
                        Ok(DbConnection { url })
                    }
                })
            },
            |_| Box::pin(async {}),
        )
        .await;
        let connection = manager
            .get_resource(Context::default(), "db_connection", 1)
            .await;

        assert_eq!(&connection.unwrap().url, url_string);
    }

    #[tokio::test]
    async fn test_idle_time_expired() {
        let manager = ContextResourceManager::new(
            |_| Box::pin(async { ("global".to_string(), 100) }),
            |_| Box::pin(async { Ok(100) }),
            |_| Box::pin(async {}),
        )
        .await;

        _ = manager.get_resource(Context::default(), "counter", 3).await;

        sleep(std::time::Duration::from_secs(6)).await;
        assert!(!(manager.has_resource("counter").await));
    }

    #[tokio::test]
    async fn test_idle_time_not_expired() {
        let manager = ContextResourceManager::new(
            |_| Box::pin(async { ("global".to_owned(), 20) }),
            |_| Box::pin(async { Ok(100) }),
            |_| Box::pin(async {}),
        )
        .await;

        _ = manager.get_resource(Context::default(), "counter", 3).await;

        assert!(manager.has_resource("counter").await);
    }
}
