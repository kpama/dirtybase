use std::{borrow::BorrowMut, collections::HashMap, sync::RwLock};

pub struct RegisterRoute {
    prefix: String,
    router: axum::Router<()>,
}

impl busstop::DispatchableCommand for RegisterRoute {}

impl RegisterRoute {
    pub fn new(prefix: &str, router: axum::Router<()>) -> Self {
        Self {
            prefix: prefix.to_string(),
            router,
        }
    }
}

#[derive(Default)]
pub(crate) struct RegisteredRoutes(RwLock<HashMap<String, Vec<axum::Router<()>>>>);

impl RegisteredRoutes {
    pub(crate) fn append(&self, prefix: &str, router: axum::Router) -> &Self {
        if let Ok(mut lock) = self.0.write() {
            if !lock.contains_key(prefix) {
                lock.insert(prefix.to_string(), Vec::new());
            }

            lock.get_mut(prefix).unwrap().push(router);
        }

        self
    }

    pub(crate) fn merge(&self, mut router: axum::Router) -> axum::Router {
        if let Ok(lock) = self.0.write() {
            for (prefix, collection) in lock.clone().into_iter() {
                for a_router in collection {
                    router = router.nest(&prefix, a_router);
                }
            }
        }
        router
    }
}

#[async_trait::async_trait]
impl busybody::Injectable for RegisteredRoutes {
    async fn inject(_: &busybody::ServiceContainer) -> Self {
        Self::default()
    }
}

#[derive(Debug, Default)]
pub(crate) struct RegisterRouteHandler;

#[async_trait::async_trait]
impl busstop::CommandHandler for RegisterRouteHandler {
    async fn handle_command(
        &self,
        mut dispatched: busstop::DispatchedCommand,
    ) -> busstop::DispatchedCommand {
        let mut routes = busybody::helpers::service_container()
            .get_or_inject::<RegisteredRoutes>()
            .await;
        if let Some(command) = dispatched.take_command::<RegisterRoute>() {
            routes.borrow_mut().append(&command.prefix, command.router);
        }

        dispatched
    }
}
