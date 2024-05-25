use std::{collections::HashMap, sync::Arc};

use named_routes_axum::RouterWrapper;

pub trait MiddlewareRegisterer {
    fn register(
        &self,
        router: RouterWrapper<Arc<busybody::ServiceContainer>>,
    ) -> RouterWrapper<Arc<busybody::ServiceContainer>>;
}

pub struct MiddlewareManager(HashMap<String, Box<dyn MiddlewareRegisterer>>);

impl MiddlewareManager {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn add(
        &mut self,
        name: &str,
        registerer: impl MiddlewareRegisterer + 'static,
    ) -> &mut Self {
        self.0.insert(name.into(), Box::new(registerer));
        self
    }

    pub fn register(
        &mut self,
        mut router: RouterWrapper<Arc<busybody::ServiceContainer>>,
        order: impl IntoIterator<Item = String>,
    ) -> RouterWrapper<Arc<busybody::ServiceContainer>> {
        for m in order.into_iter() {
            if self.0.contains_key(&m) {
                dbg!("about to register middleware: {}", &m);
                router = MiddlewareRegisterer::register(self.0.get(&m).unwrap().as_ref(), router);
            }
        }

        router
    }
}
