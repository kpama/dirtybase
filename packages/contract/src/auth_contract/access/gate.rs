use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use busybody::{Handler, Resolver, ServiceContainer};
use futures::future::BoxFuture;
use tokio::sync::RwLock;

use crate::prelude::Context;

use super::{
    gate_after_middleware::GateAfterMiddleware, gate_before_middleware::GateBeforeMiddleware,
    GateResponse,
};

type GateCollection = HashMap<
    String,
    Arc<
        Box<
            dyn Fn(ServiceContainer) -> BoxFuture<'static, Option<GateResponse>>
                + Send
                + Sync
                + 'static,
        >,
    >,
>;

pub(crate) static GATE_COLLECTION: OnceLock<RwLock<GateCollection>> = OnceLock::new();
pub struct Gate {
    sc: ServiceContainer,
}

impl Default for Gate {
    fn default() -> Self {
        Self::new()
    }
}

impl Gate {
    pub fn new() -> Self {
        Self {
            sc: busybody::helpers::make_task_proxy_or_fallback(),
        }
    }

    /// Register a new permission handler
    pub async fn define<F, R, Args>(name: &str, handler: F)
    where
        F: Clone + Handler<Args, Output = Option<R>> + Send + Sync + 'static,
        Args: Clone + Resolver + 'static + Send,
        <F as busybody::Handler<Args>>::Future: Send + Sync,
        R: Into<GateResponse>,
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
                    let result = cc.resolve_and_call(h.clone()).await;
                    if result.is_some() {
                        Some(result.unwrap().into())
                    } else {
                        None
                    }
                })
            })),
        );
    }

    pub async fn before<F, R, Args>(before: F)
    where
        F: Clone + Handler<Args, Output = Option<R>> + Send + Sync + 'static,
        Args: Clone + Resolver + 'static + Send,
        <F as busybody::Handler<Args>>::Future: Send + Sync,
        R: Into<GateResponse>,
    {
        GateBeforeMiddleware::register(move |resolver| {
            //
            let cb = before.clone();
            async move {
                let result = resolver.sc.resolve_and_call(cb).await;
                if result.is_some() {
                    return Some(result.unwrap().into());
                }
                None
            }
        })
        .await;
    }

    pub async fn after<F, R, Args>(after: F)
    where
        F: Clone + Handler<Args, Output = Option<R>> + Send + Sync + 'static,
        Args: Clone + Resolver + 'static + Send,
        <F as busybody::Handler<Args>>::Future: Send + Sync,
        R: Into<GateResponse>,
    {
        GateAfterMiddleware::register(move |resolver| {
            let cb = after.clone();
            async move {
                let result = resolver.sc.resolve_and_call(cb).await;
                if result.is_some() {
                    return Some(result.unwrap().into());
                }
                None
            }
        })
        .await;
    }

    pub async fn response(&self, name: &str) -> GateResponse {
        let result = GateBeforeMiddleware::new(self.sc.clone(), name)
            .handle()
            .await;
        if result.is_some() {
            return result.unwrap();
        }
        if let Some(rw_lock) = GATE_COLLECTION.get() {
            let r_lock = rw_lock.read().await;
            if let Some(callback) = r_lock.get(name) {
                let result = callback(self.sc.clone()).await;
                if result.is_some() {
                    return result.unwrap();
                }
            }
        }

        let result = GateAfterMiddleware::new(self.sc.clone(), name)
            .handle()
            .await;
        if result.is_some() {
            return result.unwrap();
        }

        GateResponse::deny()
    }

    pub async fn response_when<P: Clone + Send + Sync + 'static>(
        &self,
        name: &str,
        params: P,
    ) -> GateResponse {
        let sc = if self.sc.is_task_proxy() {
            let p_ci = busybody::helpers::make_proxy();
            p_ci.set_type(params).await;
            p_ci
        } else {
            self.sc.set_type::<P>(params).await;
            self.sc.clone()
        };

        let result = Gate::from(sc).response(name).await;
        self.sc.forget::<P>().await;

        result
    }

    pub async fn allows(&self, name: &str) -> bool {
        self.response(name).await == true
    }

    pub async fn can(&self, name: &str) -> bool {
        self.allows(name).await
    }

    pub async fn cannot(&self, name: &str) -> bool {
        !self.allows(name).await
    }

    pub async fn all(&self, names: &[&str]) -> bool {
        for name in names {
            if !self.allows(*name).await {
                return false;
            }
        }
        true
    }

    pub async fn any(&self, names: &[&str]) -> bool {
        for name in names {
            if self.allows(*name).await {
                return true;
            }
        }
        false
    }
    pub async fn any_when<P: Clone + Send + Sync + 'static>(
        &self,
        names: &[&str],
        params: P,
    ) -> bool {
        for name in names {
            if self.allows_when(*name, params.clone()).await {
                return true;
            }
        }
        false
    }

    pub async fn denies(&self, name: &str) -> bool {
        self.allows(name).await == false
    }

    pub async fn denies_when<P: Clone + Send + Sync + 'static>(
        &self,
        name: &str,
        params: P,
    ) -> bool {
        !self.allows_when(name, params).await
    }

    pub async fn check(&self, names: &[&str]) -> bool {
        self.all(names).await
    }

    pub async fn check_when<P: Clone + Send + Sync + 'static>(
        &self,
        names: &[&str],
        params: P,
    ) -> bool {
        self.all_when(names, params).await
    }

    pub async fn all_when<P: Clone + Send + Sync + 'static>(
        &self,
        names: &[&str],
        params: P,
    ) -> bool {
        for name in names {
            if !self.allows_when(name, params.clone()).await {
                return false;
            }
        }

        true
    }

    pub async fn allows_when<P: Clone + Send + Sync + 'static>(
        &self,
        name: &str,
        params: P,
    ) -> bool {
        self.response_when(name, params).await == true
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
