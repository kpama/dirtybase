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
    param: MiddlewareParam,
}

impl Registerer {
    /// Register a middleware
    pub fn middleware<F, Fut, Out>(mut self, mut handler: F) -> Self
    where
        F: FnMut(Request, MiddlewareParam, Next) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Out> + Send + 'static,
        Out: IntoResponse + 'static,
    {
        let name = self.param.name();
        let params = self.param.clone();
        self.wrapper = self.wrapper.middleware(move |req, next| {
            //
            let name = name.clone();
            let result = (handler)(req, params.clone(), next);
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
        F: FnMut(State<ST>, Request, MiddlewareParam, Next) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Out> + Send + 'static,
        Out: IntoResponse + 'static,
        ST: Clone + Send + Sync + 'static,
    {
        let name = self.param.name();
        let params = self.param.clone();

        self.wrapper = self.wrapper.middleware_with_state(
            move |state, req, next| {
                //
                let name = name.clone();
                let result = (handler)(state, req, params.clone(), next);
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
        F: FnMut(Request, MiddlewareParam, Next) -> Fut + Clone + Send + Sync + 'static,
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
        F: FnMut(State<ST>, Request, MiddlewareParam, Next) -> Fut + Clone + Send + Sync + 'static,
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
            let param = MiddlewareParam::from(name);
            if let Some(m) = self.0.get(param.name_ref()) {
                let mut reg = Registerer {
                    wrapper: router,
                    param,
                };
                reg = (m)(reg);
                router = reg.inner();
            } else {
                // FIXME: Add translation
                tracing::error!("could not find web middleware: {}", param.name_ref());
            }
        }

        router
    }
}

#[derive(Debug, Clone)]
pub struct MiddlewareParam {
    name: Arc<String>,
    kind: Arc<String>,
    args: Arc<HashMap<String, String>>,
}

impl MiddlewareParam {
    pub fn new(name: String, kind: String, args: HashMap<String, String>) -> Self {
        Self {
            name: Arc::new(name),
            kind: Arc::new(kind),
            args: Arc::new(args),
        }
    }

    pub fn kind_ref(&self) -> &str {
        self.kind.as_str()
    }
    pub fn kind(&self) -> Arc<String> {
        self.kind.clone()
    }

    pub fn name_ref(&self) -> &str {
        self.name.as_str()
    }

    pub fn name(&self) -> Arc<String> {
        self.name.clone()
    }

    pub fn arg(&self, name: &str) -> Option<String> {
        self.args.get(name).cloned()
    }

    pub fn has(&self, name: &str) -> bool {
        self.args.contains_key(name)
    }
}

// parses "name::kind>arg1=v1,arg2=v2" to an Instance
impl From<String> for MiddlewareParam {
    fn from(subject: String) -> Self {
        let (name, rest) = if subject.contains("::") {
            subject.split_once("::").unwrap_or_default()
        } else {
            (subject.as_str(), "")
        };
        let (kind, args_str) = rest.split_once(">").unwrap_or((rest, ""));
        let args = args_str
            .split(",")
            .map(|e| e.split_once("=").unwrap_or_default())
            .filter(|x| !x.0.is_empty() && !x.1.is_empty())
            .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
            .collect::<HashMap<String, String>>();

        MiddlewareParam::new(name.trim().to_string(), kind.trim().to_string(), args)
    }
}

impl From<&str> for MiddlewareParam {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}
