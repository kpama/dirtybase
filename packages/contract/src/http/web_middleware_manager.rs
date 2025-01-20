use std::{collections::HashMap, sync::Arc};

use named_routes_axum::RouterWrapper;

pub type WrappedRouter = RouterWrapper<Arc<busybody::ServiceContainer>>;

pub trait WebMiddlewareRegisterer: Send + Sync {
    fn register(&self, router: WrappedRouter) -> WrappedRouter;
}

pub struct WebMiddlewareManager(HashMap<String, Box<dyn WebMiddlewareRegisterer>>);

impl Default for WebMiddlewareManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WebMiddlewareManager {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Register a middleware that maybe apply later
    pub fn register(
        &mut self,
        name: &str,
        registerer: impl WebMiddlewareRegisterer + 'static,
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
                router =
                    WebMiddlewareRegisterer::register(self.0.get(&key).unwrap().as_ref(), router);
            }
        }

        router
    }
}
