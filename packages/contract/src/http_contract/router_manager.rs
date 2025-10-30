use std::collections::HashMap;

use super::{RouterBuilder, WrappedRouter};

pub type ExtensionRouter = WrappedRouter;

#[derive(Debug, PartialEq, PartialOrd, Eq, Hash)]
pub enum RouteType {
    Api,
    InsecureApi,
    Backend,
    General,
    Dev,
}

pub struct RouterManager {
    builders: HashMap<RouteType, (String, Option<RouterBuilder>)>,
}

impl RouterManager {
    pub fn new(
        api_prefix: &str,
        backend_prefix: &str,
        insecure_api_prefix: &str,
        dev_prefix: &str,
    ) -> Self {
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

        let mut builders = HashMap::new();
        // general
        builders.insert(RouteType::General, ("".to_string(), None));
        // API
        builders.insert(RouteType::Api, (api, None));
        // insecure API
        builders.insert(RouteType::InsecureApi, (insecure_api, None));
        // backend
        builders.insert(RouteType::Backend, (backend, None));
        // dev
        builders.insert(RouteType::Dev, (dev, None));

        Self { builders }
    }

    pub fn api(
        &mut self,
        prefix: Option<&str>,
        mut callback: impl FnMut(&mut RouterBuilder),
    ) -> &mut Self {
        let mut builder = RouterBuilder::new(Some(&self.generate_prefix(RouteType::Api, prefix)));
        callback(&mut builder);
        self.append(RouteType::Api, prefix.unwrap_or_default(), builder)
    }

    pub fn insecure_api(
        &mut self,
        prefix: Option<&str>,
        mut callback: impl FnMut(&mut RouterBuilder),
    ) -> &mut Self {
        let mut builder =
            RouterBuilder::new(Some(&self.generate_prefix(RouteType::InsecureApi, prefix)));
        callback(&mut builder);
        self.append(RouteType::InsecureApi, prefix.unwrap_or_default(), builder)
    }

    pub fn backend(
        &mut self,
        prefix: Option<&str>,
        mut callback: impl FnMut(&mut RouterBuilder),
    ) -> &mut Self {
        let mut builder =
            RouterBuilder::new(Some(&self.generate_prefix(RouteType::Backend, prefix)));
        callback(&mut builder);
        self.append(RouteType::Backend, prefix.unwrap_or_default(), builder)
    }

    pub fn general(
        &mut self,
        prefix: Option<&str>,
        mut callback: impl FnMut(&mut RouterBuilder),
    ) -> &mut Self {
        let mut builder =
            RouterBuilder::new(Some(&self.generate_prefix(RouteType::General, prefix)));
        callback(&mut builder);
        self.append(RouteType::General, prefix.unwrap_or_default(), builder)
    }

    pub fn dev(
        &mut self,
        prefix: Option<&str>,
        mut callback: impl FnMut(&mut RouterBuilder),
    ) -> &mut Self {
        let mut builder = RouterBuilder::new(Some(&self.generate_prefix(RouteType::Dev, prefix)));
        callback(&mut builder);
        self.append(RouteType::Dev, prefix.unwrap_or_default(), builder)
    }

    pub fn take(self) -> HashMap<RouteType, (String, Option<RouterBuilder>)> {
        self.builders
    }

    fn append(&mut self, base_type: RouteType, prefix: &str, builder: RouterBuilder) -> &mut Self {
        if let Some(entry) = self.builders.get_mut(&base_type) {
            if entry.1.is_none() {
                entry.1 = Some(RouterBuilder::new(Some(&entry.0)));
            }

            entry.1.as_mut().unwrap().append(builder, prefix);
        }

        self
    }

    fn generate_prefix(&self, base_type: RouteType, prefix: Option<&str>) -> String {
        format!(
            "{}{}",
            if let Some(entry) = self.builders.get(&base_type) {
                &entry.0
            } else {
                ""
            },
            prefix.unwrap_or_default()
        )
    }
}
