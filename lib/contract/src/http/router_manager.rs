use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct RouteCollection {
    pub prefix: String,
    pub base_route: axum::Router<Arc<busybody::ServiceContainer>>,
    pub routers: HashMap<String, Vec<axum::Router<Arc<busybody::ServiceContainer>>>>,
}

impl RouteCollection {
    pub(crate) fn new(prefix: String) -> Self {
        let base_route = axum::Router::new();
        Self {
            prefix: prefix.to_string(),
            routers: HashMap::new(),
            base_route,
        }
    }

    pub fn prefix(&self) -> String {
        self.prefix.clone()
    }

    pub fn add(&mut self, prefix: &str, router: axum::Router<Arc<busybody::ServiceContainer>>) {
        let full_path = format!("{}{}", &self.prefix, prefix);
        if !self.routers.contains_key(&full_path) {
            self.routers.insert(full_path, vec![router]);
        } else {
            self.routers.get_mut(&full_path).unwrap().push(router);
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Hash)]
pub enum RouteType {
    Api,
    Backend,
    General,
}

pub struct RouterManager {
    base: HashMap<RouteType, RouteCollection>,
}

impl RouterManager {
    pub fn new(api_prefix: Option<&str>, backend_prefix: Option<&str>) -> Self {
        let mut routers = HashMap::new();

        let api = api_prefix.unwrap_or_else(|| "/api").to_string();
        let backend = backend_prefix.unwrap_or_else(|| "/_admin").to_string();

        // general
        routers.insert(RouteType::General, RouteCollection::new("".to_string()));
        // api
        routers.insert(RouteType::Api, RouteCollection::new(api));

        // backend
        routers.insert(RouteType::Backend, RouteCollection::new(backend));

        Self { base: routers }
    }

    pub fn api(
        &mut self,
        prefix: &str,
        router: axum::Router<Arc<busybody::ServiceContainer>>,
    ) -> &mut Self {
        self.append(RouteType::Api, prefix, router)
    }

    pub fn backend(
        &mut self,
        prefix: &str,
        router: axum::Router<Arc<busybody::ServiceContainer>>,
    ) -> &mut Self {
        self.append(RouteType::Backend, prefix, router)
    }

    pub fn general(
        &mut self,
        prefix: &str,
        router: axum::Router<Arc<busybody::ServiceContainer>>,
    ) -> &mut Self {
        self.append(RouteType::General, prefix, router)
    }

    pub fn take(self) -> HashMap<RouteType, RouteCollection> {
        self.base
    }

    fn append(
        &mut self,
        base_type: RouteType,
        prefix: &str,
        router: axum::Router<Arc<busybody::ServiceContainer>>,
    ) -> &mut Self {
        if let Some(base) = self.base.get_mut(&base_type) {
            base.add(prefix, router);
        }

        self
    }
}
