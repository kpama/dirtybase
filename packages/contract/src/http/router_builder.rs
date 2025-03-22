use std::collections::HashMap;

use axum::{handler::Handler, Router};
use named_routes_axum::RouterWrapper;

use super::WebMiddlewareManager;

pub struct RouterBuilder {
    wrapper: Option<RouterWrapper<busybody::ServiceContainer>>,
    middleware: Option<Vec<String>>,
    nest: Option<HashMap<String, RouterBuilder>>,
    merge: Option<Vec<RouterBuilder>>,
}

impl Default for RouterBuilder {
    fn default() -> Self {
        Self {
            wrapper: Some(RouterWrapper::default()),
            middleware: None,
            nest: None,
            merge: None,
        }
    }
}

impl RouterBuilder {
    pub fn new_with_wrapper(wrapper: RouterWrapper<busybody::ServiceContainer>) -> Self {
        Self {
            wrapper: Some(wrapper),
            ..Default::default()
        }
    }

    /// Register a DELETE handler
    pub fn delete<H, T>(&mut self, path: &str, handler: H, name: &str) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().delete(path, handler, name));
        self
    }

    pub fn delete_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        name: &str,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.delete("/", handler, name);
        self.append(builder, path);
        self
    }

    /// Register a DELETE handler with no name
    pub fn delete_x<H, T>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().delete_x(path, handler));
        self
    }

    pub fn delete_x_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.delete_x("/", handler);
        self.append(builder, path);
        self
    }

    /// Register a GET handler
    pub fn get<H, T>(&mut self, path: &str, handler: H, name: &str) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().get(path, handler, name));

        self
    }

    pub fn get_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        name: &str,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.get("/", handler, name);
        self.append(builder, path);
        self
    }

    /// Register a GET handler with no name
    pub fn get_x<H, T>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().get_x(path, handler));
        self
    }

    pub fn get_x_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.get_x("/", handler);
        self.append(builder, path);
        self
    }

    /// Register a HEAD handler
    pub fn head<H, T>(&mut self, path: &str, handler: H, name: &str) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().head(path, handler, name));
        self
    }
    pub fn head_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        name: &str,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.head("/", handler, name);
        self.append(builder, path);
        self
    }

    /// Register a HEAD handler with no name
    pub fn head_x<H, T>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().head_x(path, handler));
        self
    }

    pub fn head_x_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.head_x("/", handler);
        self.append(builder, path);
        self
    }

    /// Register a OPTIONS handler
    pub fn options<H, T>(&mut self, path: &str, handler: H, name: &str) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().options(path, handler, name));
        self
    }
    pub fn options_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        name: &str,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.options("/", handler, name);
        self.append(builder, path);
        self
    }

    /// Register a OPTIONS handler with no name
    pub fn options_x<H, T>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().options_x(path, handler));
        self
    }

    pub fn options_x_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.options_x("/", handler);
        self.append(builder, path);
        self
    }

    /// Register a PATCH handler
    pub fn patch<H, T>(&mut self, path: &str, handler: H, name: &str) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().patch(path, handler, name));
        self
    }

    pub fn patch_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        name: &str,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.patch("/", handler, name);
        self.append(builder, path);
        self
    }

    /// Register a PATCH handler with no name
    pub fn patch_x<H, T>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().patch_x(path, handler));
        self
    }

    pub fn patch_x_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.patch_x("/", handler);
        self.append(builder, path);
        self
    }

    /// Register a POST handler
    pub fn post<H, T>(&mut self, path: &str, handler: H, name: &str) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().post(path, handler, name));
        self
    }

    pub fn post_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        name: &str,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.post("/", handler, name);
        self.append(builder, path);
        self
    }

    /// Register a POST handler with no name
    pub fn post_x<H, T>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().post_x(path, handler));
        self
    }

    pub fn post_x_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.post_x("/", handler);
        self.append(builder, path);
        self
    }

    /// Register a PUT handler
    pub fn put<H, T>(&mut self, path: &str, handler: H, name: &str) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().put(path, handler, name));
        self
    }

    pub fn put_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        name: &str,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.put("/", handler, name);
        self.append(builder, path);
        self
    }

    /// Register a PUT handler with no name
    pub fn put_x<H, T>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().put_x(path, handler));
        self
    }

    pub fn put_x_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.put_x("/", handler);
        self.append(builder, path);
        self
    }

    /// Register a TRACE handler
    pub fn trace<H, T>(&mut self, path: &str, handler: H, name: &str) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().trace(path, handler, name));
        self
    }

    pub fn trace_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        name: &str,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.trace("/", handler, name);
        self.append(builder, path);
        self
    }

    /// Register a TRACE handler with no name
    pub fn trace_x<H, T>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().trace_x(path, handler));
        self
    }

    pub fn trace_x_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.trace_x("/", handler);
        self.append(builder, path);
        self
    }

    /// Register a named route handler that handles most of the common HTTP verbs:
    ///  - GET, POST, PUT, DELETE, PATCH , OPTIONS, TRACE
    pub fn any<H, T>(&mut self, path: &str, handler: H, name: &str) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().any(path, handler, name));
        self
    }

    pub fn any_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        name: &str,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.any("/", handler, name);
        self.append(builder, path);
        self
    }

    /// Register a route handler that handles most of the common HTTP verbs:
    ///  - GET, POST, PUT, DELETE, PATCH , OPTIONS, TRACE
    pub fn any_x<H, T>(&mut self, path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().any_x(path, handler));
        self
    }

    pub fn any_x_with_middleware<H, T, L, I>(
        &mut self,
        path: &str,
        handler: H,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.any_x("/", handler);
        self.append(builder, path);
        self
    }

    pub fn any_of<H, T, V>(&mut self, verbs: &[V], path: &str, handler: H, name: &str) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
        V: ToString,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().any_of(verbs, path, handler, name));
        self
    }

    pub fn any_of_with_middleware<H, T, L, I, V>(
        &mut self,
        verbs: &[V],
        path: &str,
        handler: H,
        name: &str,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
        V: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.any_of(verbs, "/", handler, name);
        self.append(builder, path);
        self
    }

    pub fn any_of_x<H, T, V>(&mut self, verbs: &[V], path: &str, handler: H) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        T: 'static,
        V: ToString,
    {
        let wrapper = self.wrapper.take();
        self.wrapper = Some(wrapper.unwrap().any_of_x(verbs, path, handler));
        self
    }

    pub fn any_of_x_with_middleware<H, T, L, I, V>(
        &mut self,
        verbs: &[V],
        path: &str,
        handler: H,
        list: L,
    ) -> &mut Self
    where
        H: Handler<T, busybody::ServiceContainer>,
        L: IntoIterator<Item = I>,
        T: 'static,
        I: ToString,
        V: ToString,
    {
        let mut builder = Self::default();
        builder.middleware(list);
        builder.any_of_x(verbs, "/", handler);
        self.append(builder, path);
        self
    }

    pub fn middleware<L, I>(&mut self, list: L) -> &mut Self
    where
        L: IntoIterator<Item = I>,
        I: ToString,
    {
        let list = list
            .into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>();

        if self.middleware.is_none() {
            self.middleware = Some(list);
        } else {
            if let Some(mut existing) = self.middleware.take() {
                existing.extend(list);
                self.middleware = Some(existing);
            }
        }

        self
    }

    pub fn nest<C>(&mut self, prefix: &str, mut callback: C) -> &mut Self
    where
        C: FnMut(&mut Self) -> (),
    {
        if prefix.is_empty() || prefix == "/" {
            panic!("routes in a grouped middleware must have a valid uri prefix");
        }

        let mut router = Self::default();

        callback(&mut router);
        self.append(router, prefix);

        self
    }

    pub fn merge<C>(&mut self, mut callback: C) -> &mut Self
    where
        C: FnMut(&mut Self) -> (),
    {
        let mut router = Self::default();

        callback(&mut router);
        self.append(router, "");

        self
    }

    pub fn group_with_middleware<L, I, C>(
        &mut self,
        prefix: &str,
        mut callback: C,
        list: L,
    ) -> &mut Self
    where
        L: IntoIterator<Item = I>,
        I: ToString,
        C: FnMut(&mut Self) -> (),
    {
        if prefix.is_empty() || prefix == "/" {
            panic!("routes in a grouped middleware must have a valid uri prefix");
        }

        let mut router = Self::default();
        router.middleware(list);
        callback(&mut router);
        self.append(router, prefix);
        self
    }

    pub fn into_router_wrapper(
        &mut self,
        manager: &mut WebMiddlewareManager,
    ) -> Option<RouterWrapper<busybody::ServiceContainer>> {
        if let Some(mut wrapper) = self.wrapper.take() {
            wrapper = if let Some(order) = self.middleware.take() {
                manager.apply(wrapper, order)
            } else {
                wrapper
            };

            if let Some(list) = self.merge.take() {
                for mut entry in list {
                    if let Some(result) = entry.into_router_wrapper(manager) {
                        wrapper = wrapper.merge(result);
                    }
                }
            }

            if let Some(map) = self.nest.take() {
                for (prefix, mut builder) in map {
                    if let Some(result) = builder.into_router_wrapper(manager) {
                        wrapper = wrapper.nest(&prefix, result);
                    }
                }
            }

            return Some(wrapper);
        }
        None
    }

    pub fn build(
        mut self,
        manager: &mut WebMiddlewareManager,
    ) -> Option<Router<busybody::ServiceContainer>> {
        if let Some(wrapper) = self.into_router_wrapper(manager) {
            return Some(wrapper.into_router());
        }
        None
    }

    pub(crate) fn append(&mut self, builder: Self, prefix: &str) {
        if !prefix.is_empty() {
            if self.nest.is_none() {
                let mut map = HashMap::new();
                map.insert(prefix.to_string(), builder);
                self.nest = Some(map);
            } else {
                if let Some(mut map) = self.nest.take() {
                    if let Some(existing) = map.get_mut(prefix) {
                        existing.append(builder, ""); // nested prefix already exist
                    } else {
                        map.insert(prefix.to_string(), builder);
                    }
                    self.nest = Some(map);
                }
            }
        } else {
            if self.merge.is_none() {
                self.merge = Some(vec![builder])
            } else {
                if let Some(mut list) = self.merge.take() {
                    list.push(builder);
                    self.merge = Some(list);
                }
            }
        }
    }
}
