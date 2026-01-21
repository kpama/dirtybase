use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
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
    u64,
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
    pub async fn define<F, R, Args>(ability: impl AsRef<str>, handler: F)
    where
        F: Clone + Handler<Args, Output = Option<R>> + Send + Sync + 'static,
        Args: Clone + Resolver + 'static + Send,
        <F as busybody::Handler<Args>>::Future: Send,
        R: Into<GateResponse>,
    {
        let rw_lock = GATE_COLLECTION.get_or_init(RwLock::default);
        let mut w_lock = rw_lock.write().await;
        let mut hasher = DefaultHasher::new();
        ability.as_ref().hash(&mut hasher);
        let hash = hasher.finish();

        tracing::trace!(
            "defining gate for ability: {}, hash: {}",
            &ability.as_ref(),
            hash
        );

        w_lock.insert(
            hash,
            Arc::new(Box::new(move |c| {
                let cc = c.clone();
                let h = handler.clone();
                Box::pin(async move {
                    //
                    let result = cc.resolve_and_call(h.clone()).await;
                    if let Some(r) = result {
                        return Some(r.into());
                    }
                    None
                })
            })),
        );
    }

    /// Register a callback that will be call before any gate is resolved
    /// Returning Some(T) will halt the actual guard and after callbacks from being called
    pub async fn before<F, R, Args>(before: F)
    where
        F: Clone + Handler<Args, Output = Option<R>> + Send + Sync + 'static,
        Args: Clone + Resolver + 'static + Send,
        <F as busybody::Handler<Args>>::Future: Send,
        R: Into<GateResponse>,
    {
        GateBeforeMiddleware::register(move |resolver| {
            //
            let cb = before.clone();
            async move {
                let result = resolver.sc.resolve_and_call(cb).await;
                if let Some(r) = result {
                    return Some(r.into());
                }
                None
            }
        })
        .await;
    }

    /// Registers a callback that will be called when all previous resolving fail
    pub async fn after<F, R, Args>(after: F)
    where
        F: Clone + Handler<Args, Output = Option<R>> + Send + Sync + 'static,
        Args: Clone + Resolver + 'static + Send,
        <F as busybody::Handler<Args>>::Future: Send,
        R: Into<GateResponse>,
    {
        GateAfterMiddleware::register(move |resolver| {
            let cb = after.clone();
            async move {
                let result = resolver.sc.resolve_and_call(cb).await;
                if let Some(r) = result {
                    return Some(r.into());
                }
                None
            }
        })
        .await;
    }

    /// Check the specified ability returning a `GateResponse`
    pub async fn response(&self, ability: impl AsRef<str>) -> GateResponse {
        let mut hasher = DefaultHasher::new();
        ability.as_ref().hash(&mut hasher);
        let hash = hasher.finish();

        tracing::trace!(
            "gate checking ability: {}, hash: {}",
            &ability.as_ref(),
            hash
        );

        let result = GateBeforeMiddleware::new(self.sc.clone(), hash)
            .handle()
            .await;
        if let Some(r) = result {
            return r;
        }

        if let Some(rw_lock) = GATE_COLLECTION.get() {
            let r_lock = rw_lock.read().await;
            if let Some(callback) = r_lock.get(&hash) {
                let result = callback(self.sc.clone()).await;
                if let Some(r) = result {
                    return r;
                }
            }
        }

        let result = GateAfterMiddleware::new(self.sc.clone(), hash)
            .handle()
            .await;

        if let Some(r) = result {
            return r;
        }

        GateResponse::deny()
    }

    /// Checks the specified ability when giving these parameters
    /// This method can be use when you want to override the default
    /// parameters passed to the ability's guard
    pub async fn response_when<P: Clone + Send + Sync + 'static>(
        &self,
        ability: impl AsRef<str>,
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

    /// Check if the specified ability is allows in the current context
    pub async fn allows(&self, ability: impl AsRef<str>) -> bool {
        self.response(ability).await == true
    }

    /// Check if the specified ability is allow
    pub async fn can(&self, ability: impl AsRef<str>) -> bool {
        self.allows(ability).await
    }

    /// Checks if the specified ability is not allow in the current context
    pub async fn cannot(&self, ability: impl AsRef<str>) -> bool {
        !self.allows(ability).await
    }

    /// Checks if multiple abilities are allowed in the current context.
    /// All ability must be allow or else all fail
    pub async fn all(&self, abilities: &[impl AsRef<str>]) -> bool {
        for ability in abilities {
            if !self.allows(ability).await {
                return false;
            }
        }
        true
    }

    /// Checks if one or more abilities in the slice is allow.
    /// True will be return if at least an ability is allowed.
    pub async fn any(&self, abilities: &[impl AsRef<str>]) -> bool {
        for ability in abilities {
            if self.allows(ability).await {
                return true;
            }
        }
        false
    }

    /// Checks if one or more abilities in the slice is allow.
    /// True will be return if at least an ability is allowed.
    /// The guard is provided with the parameters passed.
    pub async fn any_when<P: Clone + Send + Sync + 'static>(
        &self,
        abilities: &[&str],
        params: P,
    ) -> bool {
        for ability in abilities {
            if self.allows_when(ability, params.clone()).await {
                return true;
            }
        }
        false
    }

    /// Checks if an ability is not allowed
    pub async fn denies(&self, ability: &str) -> bool {
        !(self.allows(ability).await)
    }

    /// Checks if an ability is not allowed
    /// The provided parameters are passed to the guard
    pub async fn denies_when<P: Clone + Send + Sync + 'static>(
        &self,
        ability: impl AsRef<str>,
        params: P,
    ) -> bool {
        !self.allows_when(ability, params).await
    }

    /// Alias to `all`
    pub async fn check(&self, abilities: &[impl AsRef<str>]) -> bool {
        self.all(abilities).await
    }

    /// Alias for `allows_when`
    pub async fn check_when<P: Clone + Send + Sync + 'static>(
        &self,
        abilities: &[impl AsRef<str>],
        params: P,
    ) -> bool {
        self.all_when(abilities, params).await
    }

    /// Alias for `allows_when`
    pub async fn all_when<P: Clone + Send + Sync + 'static>(
        &self,
        abilities: &[impl AsRef<str>],
        params: P,
    ) -> bool {
        for ability in abilities {
            if !self.allows_when(ability, params.clone()).await {
                return false;
            }
        }

        true
    }

    /// Alias for `response_when`
    pub async fn allows_when<P: Clone + Send + Sync + 'static>(
        &self,
        ability: impl AsRef<str>,
        params: P,
    ) -> bool {
        self.response_when(ability, params).await == true
    }

    /// Sets an instance of type `T` into the current service container
    pub async fn set<T: Clone + Send + Sync + 'static>(&self, value: T) -> &Self {
        self.sc.set_type(value).await;
        self
    }

    /// Gets an instance of type `T` from the current service container
    pub async fn get<T: Clone + Send + Sync + 'static>(&self) -> Result<T, anyhow::Error> {
        let result = self.sc.get_type().await;

        if let Some(r) = result {
            return Ok(r);
        }

        if let Some(ctx) = self.sc.get_type::<Context>().await {
            return ctx.get().await;
        }

        Err(anyhow::anyhow!("Instance not found"))
    }
}

impl From<busybody::ServiceContainer> for Gate {
    fn from(sc: busybody::ServiceContainer) -> Self {
        Self { sc }
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
