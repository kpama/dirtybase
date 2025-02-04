use std::{collections::HashMap, sync::Arc};

use named_routes_axum::RouterWrapper;

pub type WrappedRouter = RouterWrapper<Arc<busybody::ServiceContainer>>;

type RegistererFn = Box<dyn Fn(WrappedRouter) -> WrappedRouter + Send + Sync + 'static>;

pub struct WebMiddlewareManager(HashMap<String, RegistererFn>);

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
    pub fn register<F>(&mut self, name: &str, registerer: F) -> &mut Self
    where
        F: Fn(WrappedRouter) -> WrappedRouter + Send + Sync + 'static,
    {
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
            if key.is_empty() {
                continue;
            }
            if let Some(m) = self.0.get(&key) {
                router = (m)(router);
            } else {
                // FIXME: Add translation
                tracing::error!("could not find web middleware: {}", key);
            }
        }

        router
    }
}
