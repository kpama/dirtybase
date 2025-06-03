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

#[derive(Debug, Clone)]
pub struct Gate {
    sc: ServiceContainer,
}

impl Gate {
    pub fn new(sc: ServiceContainer) -> Self {
        Self { sc }
    }

    /// Register a new permission handler
    pub async fn define<F, R, Args>(ability: &str, handler: F)
    where
        F: Clone + Handler<Args, Output = Option<R>> + Send + Sync + 'static,
        Args: Clone + Resolver + 'static + Send,
        <F as busybody::Handler<Args>>::Future: Send + Sync,
        R: Into<GateResponse>,
    {
        let rw_lock = GATE_COLLECTION.get_or_init(|| RwLock::default());
        let mut w_lock = rw_lock.write().await;
        w_lock.insert(
            ability.to_string(),
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

    pub async fn response(&self, ability: &str) -> GateResponse {
        let result = GateBeforeMiddleware::new(self.sc.clone(), ability)
            .handle()
            .await;
        if result.is_some() {
            return result.unwrap();
        }
        if let Some(rw_lock) = GATE_COLLECTION.get() {
            let r_lock = rw_lock.read().await;
            if let Some(callback) = r_lock.get(ability) {
                let result = callback(self.sc.clone()).await;
                if result.is_some() {
                    return result.unwrap();
                }
            }
        }

        let result = GateAfterMiddleware::new(self.sc.clone(), ability)
            .handle()
            .await;
        if result.is_some() {
            return result.unwrap();
        }

        GateResponse::deny()
    }

    pub async fn response_when<P: Clone + Send + Sync + 'static>(
        &self,
        ability: &str,
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

        let result = Gate::from(sc).response(ability).await;
        self.sc.forget::<P>().await;

        result
    }

    pub async fn allows(&self, ability: &str) -> bool {
        self.response(ability).await == true
    }

    pub async fn can(&self, ability: &str) -> bool {
        self.allows(ability).await
    }

    pub async fn cannot(&self, ability: &str) -> bool {
        !self.allows(ability).await
    }

    pub async fn all(&self, abilities: &[&str]) -> bool {
        for ability in abilities {
            if !self.allows(*ability).await {
                return false;
            }
        }
        true
    }

    pub async fn any(&self, abilities: &[&str]) -> bool {
        for ability in abilities {
            if self.allows(*ability).await {
                return true;
            }
        }
        false
    }
    pub async fn any_when<P: Clone + Send + Sync + 'static>(
        &self,
        abilities: &[&str],
        params: P,
    ) -> bool {
        for ability in abilities {
            if self.allows_when(*ability, params.clone()).await {
                return true;
            }
        }
        false
    }

    pub async fn denies(&self, ability: &str) -> bool {
        self.allows(ability).await == false
    }

    pub async fn denies_when<P: Clone + Send + Sync + 'static>(
        &self,
        ability: &str,
        params: P,
    ) -> bool {
        !self.allows_when(ability, params).await
    }

    pub async fn check(&self, abilities: &[&str]) -> bool {
        self.all(abilities).await
    }

    pub async fn check_when<P: Clone + Send + Sync + 'static>(
        &self,
        abilities: &[&str],
        params: P,
    ) -> bool {
        self.all_when(abilities, params).await
    }

    pub async fn all_when<P: Clone + Send + Sync + 'static>(
        &self,
        abilities: &[&str],
        params: P,
    ) -> bool {
        for ability in abilities {
            if !self.allows_when(ability, params.clone()).await {
                return false;
            }
        }

        true
    }

    pub async fn allows_when<P: Clone + Send + Sync + 'static>(
        &self,
        ability: &str,
        params: P,
    ) -> bool {
        self.response_when(ability, params).await == true
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

impl From<Context> for Gate {
    fn from(value: Context) -> Self {
        Self::new(value.container())
    }
}

impl From<&Context> for Gate {
    fn from(value: &Context) -> Self {
        Self::new(value.container())
    }
}
