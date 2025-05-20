use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use busybody::{Handler, Resolver, ServiceContainer};
use dirtybase_contract::prelude::Context;
use futures::future::BoxFuture;
use tokio::sync::RwLock;

type GateCollection = HashMap<
    String,
    Arc<Box<dyn Fn(ServiceContainer) -> BoxFuture<'static, bool> + Send + Sync + 'static>>,
>;

pub(crate) static GATE_COLLECTION: OnceLock<RwLock<GateCollection>> = OnceLock::new();
pub struct Gate {
    sc: ServiceContainer,
}

impl Gate {
    pub async fn new() -> Self {
        Self {
            sc: busybody::helpers::make_task_proxy_or_fallback(),
        }
    }

    pub async fn define<F, Args>(name: &str, handler: F)
    where
        F: Clone + Handler<Args, Output = bool> + Send + Sync + 'static,
        Args: Clone + Resolver + 'static + Send,
        <F as busybody::Handler<Args>>::Future: Send + Sync,
    {
        let rw_lock = GATE_COLLECTION.get_or_init(|| RwLock::default());
        let mut w_lock = rw_lock.write().await;
        w_lock.insert(
            name.to_string(),
            Arc::new(Box::new(move |c| {
                let cc = c.clone();
                let h = handler.clone();
                Box::pin(async move {
                    //
                    cc.resolve_and_call(h.clone()).await
                })
            })),
        );
    }

    pub async fn allows(&mut self, name: &str) -> bool {
        if let Some(rw_lock) = GATE_COLLECTION.get() {
            let r_lock = rw_lock.read().await;
            if let Some(callback) = r_lock.get(name) {
                return callback(self.sc.clone()).await;
            }
        }

        false
    }

    pub async fn allows_when<P: Clone + Send + Sync + 'static>(
        &mut self,
        name: &str,
        params: P,
    ) -> bool {
        if let Some(rw_lock) = GATE_COLLECTION.get() {
            let r_lock = rw_lock.read().await;
            if let Some(callback) = r_lock.get(name) {
                let p_ci = busybody::helpers::make_proxy();
                p_ci.set_type(params).await;
                return callback(p_ci).await;
            }
        }

        false
    }

    pub async fn set<T: Clone + Send + Sync + 'static>(&self, value: T) -> &Self {
        self.sc.set_type(value).await;
        self
    }

    pub async fn get<T: Clone + Send + Sync + 'static>(&self) -> Result<T, anyhow::Error> {
        let result = self.sc.get_type().await;

        if result.is_some() {
            return Ok(result.unwrap());
        }

        if let Some(ctx) = self.sc.get_type::<Context>().await {
            return ctx.get().await;
        }

        Err(anyhow::anyhow!("Instance not found"))
    }
}

impl From<busybody::ServiceContainer> for Gate {
    fn from(sc: busybody::ServiceContainer) -> Self {
        Self { sc: sc }
    }
}
