use std::{collections::HashMap, ops::Deref, sync::Arc};

use named_routes_axum::RouterWrapper;

pub type WrappedRouter = RouterWrapper<Arc<busybody::ServiceContainer>>;

pub trait MiddlewareRegisterer: Send + Sync {
    fn register(&self, router: WrappedRouter) -> WrappedRouter;
}

pub struct MiddlewareManager(HashMap<String, Box<dyn MiddlewareRegisterer>>);

impl Default for MiddlewareManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MiddlewareManager {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Register a middleware that maybe apply later
    pub fn register(
        &mut self,
        name: &str,
        registerer: impl MiddlewareRegisterer + 'static,
    ) -> &mut Self {
        self.0.insert(name.into(), Box::new(registerer));
        self
    }

    /// Apply the specified middlewares on the router
    pub fn apply<I>(
        &self,
        mut router: RouterWrapper<Arc<busybody::ServiceContainer>>,
        order: impl IntoIterator<Item = I>,
    ) -> RouterWrapper<Arc<busybody::ServiceContainer>>
    where
        I: Into<String>,
    {
        for m in order.into_iter() {
            let key = m.into();
            if self.0.contains_key(&key) {
                router = MiddlewareRegisterer::register(self.0.get(&key).unwrap().as_ref(), router);
            }
        }

        router
    }
}
