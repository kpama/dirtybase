use std::{collections::HashMap, sync::Arc};

use named_routes_axum::RouterWrapper;

use super::WrappedRouter;

pub type ExtensionRouter = WrappedRouter;

pub struct RouteCollection {
    pub prefix: String,
    pub base_route: RouterWrapper<Arc<busybody::ServiceContainer>>,
    pub routers: HashMap<String, Vec<RouterWrapper<Arc<busybody::ServiceContainer>>>>,
}

impl RouteCollection {
    pub(crate) fn new(prefix: String) -> Self {
        let base_route = RouterWrapper::new();
        Self {
            prefix: prefix.to_string(),
            routers: HashMap::new(),
            base_route,
        }
    }

    pub fn prefix(&self) -> String {
        self.prefix.clone()
    }

    pub fn add(&mut self, prefix: &str, router: RouterWrapper<Arc<busybody::ServiceContainer>>) {
        let full_path = format!("{}{}", self.prefix(), prefix);
        self.routers.entry(full_path).or_default().push(router);
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Hash)]
pub enum RouteType {
    Api,
    InsecureApi,
    Backend,
    General,
}

pub struct RouterManager {
    base: HashMap<RouteType, RouteCollection>,
}

impl RouterManager {
    pub fn new(
        api_prefix: Option<&str>,
        backend_prefix: Option<&str>,
        insecure_api_prefix: Option<&str>,
    ) -> Self {
        let mut routers = HashMap::new();

        let api = api_prefix.unwrap_or("/api").to_string();
        let insecure_api = insecure_api_prefix.unwrap_or("/_open").to_string();
        let backend = backend_prefix.unwrap_or("/_admin").to_string();

        // general
        routers.insert(RouteType::General, RouteCollection::new("".to_string()));
        // api
        routers.insert(RouteType::Api, RouteCollection::new(api));
        // insecure api
        routers.insert(RouteType::InsecureApi, RouteCollection::new(insecure_api));
        // backend
        routers.insert(RouteType::Backend, RouteCollection::new(backend));

        Self { base: routers }
    }

    pub fn api(
        &mut self,
        prefix: Option<&str>,
        mut callback: impl FnMut(ExtensionRouter) -> ExtensionRouter,
    ) -> &mut Self {
        let router = callback(RouterWrapper::new());
        self.append(RouteType::Api, prefix.unwrap_or_default(), router)
    }

    pub fn insecure_api(
        &mut self,
        prefix: Option<&str>,
        callback: fn(ExtensionRouter) -> ExtensionRouter,
    ) -> &mut Self {
        let router = callback(RouterWrapper::new());
        self.append(RouteType::InsecureApi, prefix.unwrap_or_default(), router)
    }

    pub fn backend(
        &mut self,
        prefix: Option<&str>,
        mut callback: impl FnMut(ExtensionRouter) -> ExtensionRouter,
    ) -> &mut Self {
        let router = callback(RouterWrapper::new());
        self.append(RouteType::Backend, prefix.unwrap_or_default(), router)
    }

    pub fn general(
        &mut self,
        prefix: Option<&str>,
        mut callback: impl FnMut(ExtensionRouter) -> ExtensionRouter,
    ) -> &mut Self {
        let router = callback(RouterWrapper::new());
        self.append(RouteType::General, prefix.unwrap_or_default(), router)
    }

    pub fn take(self) -> HashMap<RouteType, RouteCollection> {
        self.base
    }

    fn append(
        &mut self,
        base_type: RouteType,
        prefix: &str,
        router: RouterWrapper<Arc<busybody::ServiceContainer>>,
    ) -> &mut Self {
        if let Some(base) = self.base.get_mut(&base_type) {
            base.add(prefix, router);
        }

        self
    }
}
