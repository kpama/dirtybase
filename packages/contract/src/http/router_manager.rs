use std::collections::HashMap;

use named_routes_axum::RouterWrapper;

use super::WrappedRouter;

pub type ExtensionRouter = WrappedRouter;

pub struct RouteCollection {
    pub prefix: String,
    pub base_route: RouterWrapper<busybody::ServiceContainer>,
    pub routers: HashMap<String, Vec<RouterWrapper<busybody::ServiceContainer>>>,
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

    pub fn add(&mut self, prefix: &str, router: RouterWrapper<busybody::ServiceContainer>) {
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
    Dev,
}

pub struct RouterManager {
    base: HashMap<RouteType, RouteCollection>,
}

impl RouterManager {
    pub fn new(
        api_prefix: &str,
        backend_prefix: &str,
        insecure_api_prefix: &str,
        dev_prefix: &str,
    ) -> Self {
        let mut routers = HashMap::new();

        let api = if !api_prefix.is_empty() {
            api_prefix.to_string()
        } else {
            String::from("/api")
        };

        let dev = if !dev_prefix.is_empty() {
            dev_prefix.to_string()
        } else {
            String::from("/_dev")
        };
        let insecure_api = if !insecure_api_prefix.is_empty() {
            insecure_api_prefix.to_string()
        } else {
            String::from("/_open")
        };

        let backend = if !backend_prefix.is_empty() {
            backend_prefix.to_string()
        } else {
            String::from("/_admin")
        };

        // general
        routers.insert(RouteType::General, RouteCollection::new("".to_string()));
        // api
        routers.insert(RouteType::Api, RouteCollection::new(api));
        // insecure api
        routers.insert(RouteType::InsecureApi, RouteCollection::new(insecure_api));
        // backend
        routers.insert(RouteType::Backend, RouteCollection::new(backend));
        // dev
        routers.insert(RouteType::Dev, RouteCollection::new(dev));

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

    pub fn dev(
        &mut self,
        prefix: Option<&str>,
        mut callback: impl FnMut(ExtensionRouter) -> ExtensionRouter,
    ) -> &mut Self {
        let router = callback(RouterWrapper::new());
        self.append(RouteType::Dev, prefix.unwrap_or_default(), router)
    }

    pub fn take(self) -> HashMap<RouteType, RouteCollection> {
        self.base
    }

    fn append(
        &mut self,
        base_type: RouteType,
        prefix: &str,
        router: RouterWrapper<busybody::ServiceContainer>,
    ) -> &mut Self {
        if let Some(base) = self.base.get_mut(&base_type) {
            base.add(prefix, router);
        }

        self
    }
}
