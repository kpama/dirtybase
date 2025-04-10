use std::{collections::HashMap, future::Future, sync::Arc};

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use named_routes_axum::RouterWrapper;

pub type WrappedRouter = RouterWrapper<busybody::ServiceContainer>;

type RegistererFn = Box<dyn Fn(Registerer) -> Registerer + Send + Sync>;

pub struct Registerer {
    wrapper: WrappedRouter,
    name: Arc<String>,
    params: Option<HashMap<String, String>>,
}

impl Registerer {
    /// Register a middleware
    pub fn middleware<F, Fut, Out>(mut self, mut handler: F) -> Self
    where
        F: FnMut(Request, Next, Option<HashMap<String, String>>) -> Fut
            + Clone
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = Out> + Send + 'static,
        Out: IntoResponse + 'static,
    {
        let name = self.name.clone();
        let params = self.params.take();
        self.wrapper = self.wrapper.middleware(move |req, next| {
            //
            let name = name.clone();
            let result = (handler)(req, next, params.clone());
            async move {
                tracing::trace!("calling middleware: {}", name.clone());
                let resp = result.await;
                tracing::trace!("called middleware: {}", name.clone());
                resp
            }
        });
        self
    }

    /// Register a middleware with a state
    pub fn middleware_with_state<F, Fut, Out, ST>(mut self, mut handler: F, state: ST) -> Self
    where
        F: FnMut(State<ST>, Request, Next, Option<HashMap<String, String>>) -> Fut
            + Clone
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = Out> + Send + 'static,
        Out: IntoResponse + 'static,
        ST: Clone + Send + Sync + 'static,
    {
        let name = self.name.clone();
        let params = self.params.take();

        self.wrapper = self.wrapper.middleware_with_state(
            move |state, req, next| {
                //
                let name = name.clone();
                let result = (handler)(state, req, next, params.clone());
                async move {
                    tracing::trace!("calling middleware: {}", name.clone());
                    let resp = result.await;
                    tracing::trace!("called middleware: {}", name.clone());
                    resp
                }
            },
            state,
        );

        self
    }

    pub(crate) fn inner(self) -> WrappedRouter {
        self.wrapper
    }
}

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

    pub fn register<F, Fut, Out>(&mut self, name: &str, handler: F) -> &mut Self
    where
        F: FnMut(Request, Next, Option<HashMap<String, String>>) -> Fut
            + Clone
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = Out> + Send + 'static,
        Out: IntoResponse + 'static,
    {
        self.0.insert(
            name.into(),
            Box::new(move |r| {
                //
                r.middleware(handler.clone())
            }),
        );
        self
    }

    pub fn register_with_state<F, Fut, Out, ST>(
        &mut self,
        name: &str,
        handler: F,
        state: ST,
    ) -> &mut Self
    where
        F: FnMut(State<ST>, Request, Next, Option<HashMap<String, String>>) -> Fut
            + Clone
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = Out> + Send + 'static,
        Out: IntoResponse + 'static,
        ST: Clone + Send + Sync + 'static,
    {
        self.0.insert(
            name.into(),
            Box::new(move |r| {
                //
                r.middleware_with_state(handler.clone(), state.clone())
            }),
        );

        self
    }

    /// Apply the specified middlewares on the router
    pub fn apply<I>(
        &self,
        mut router: RouterWrapper<busybody::ServiceContainer>,
        order: impl IntoIterator<Item = I>,
    ) -> RouterWrapper<busybody::ServiceContainer>
    where
        I: ToString,
    {
        for m in order.into_iter() {
            let name = m.to_string();
            if name.is_empty() {
                continue;
            }
            let (key, params) = self.split_name_and_param(name);
            if let Some(m) = self.0.get(&key) {
                let mut reg = Registerer {
                    wrapper: router,
                    name: Arc::new(key),
                    params,
                };
                reg = (m)(reg);
                router = reg.inner();
            } else {
                // FIXME: Add translation
                tracing::error!("could not find web middleware: {}", &key);
            }
        }

        router
    }

    fn split_name_and_param(&self, subject: String) -> (String, Option<HashMap<String, String>>) {
        let mut pieces = subject.split("::"); // name and params separator
        let name = pieces.next().unwrap_or_default().to_string();
        let mut params = None;
        if let Some(arg) = pieces.next() {
            params = Some(
                arg.split(":") // params separator
                    .map(|e| {
                        let mut p = e.split("="); // param and value separator
                        (
                            p.next().unwrap_or_default().to_owned(),
                            p.next().unwrap_or_default().to_owned(),
                        )
                    })
                    .collect::<HashMap<String, String>>(),
            )
        }
        (name, params)
    }
}
