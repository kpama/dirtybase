use axum::handler::Handler;
use named_routes_axum::RouterWrapper;

pub struct RouterBuilder {
    wrapper: Option<RouterWrapper<busybody::ServiceContainer>>,
    middleware: Option<Vec<String>>,
    nest: Option<Vec<RouterBuilder>>,
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

    pub fn group_with_middleware<L, I, C>(&mut self, list: L, mut callback: C) -> &mut Self
    where
        L: IntoIterator<Item = I>,
        I: ToString,
        C: FnMut(&mut Self) -> (),
    {
        let mut router = Self::default();
        router.middleware(list);

        callback(&mut router);

        if self.nest.is_none() {
            self.nest = Some(vec![router])
        } else {
            if let Some(mut list) = self.nest.take() {
                list.push(router);
                self.nest = Some(list);
            }
        }

        self
    }

    pub fn into_router_wrapper(&mut self) -> Option<RouterWrapper<busybody::ServiceContainer>> {
        self.wrapper.take()
    }

    pub fn take_nested(&mut self) -> Option<Vec<Self>> {
        self.nest.take()
    }

    pub fn take_merged(&mut self) -> Option<Vec<Self>> {
        self.merge.take()
    }
}
