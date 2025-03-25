use std::sync::Arc;

use dirtybase_contract::{
    auth::{AuthUser, AuthUserStorageProvider},
    axum::response::Response,
    prelude::Request,
};

pub struct GuardResolver {
    pub(crate) req: Request,
    pub(crate) user: Option<Result<Option<AuthUser>, anyhow::Error>>,
    pub(crate) storage: AuthUserStorageProvider,
    pub(crate) resp: Option<Response>,
    name: Arc<String>,
}

impl GuardResolver {
    pub fn new(req: Request, storage: AuthUserStorageProvider, name: &str) -> Self {
        Self {
            req,
            storage,
            user: None,
            name: name.to_string().into(),
            resp: None,
        }
    }

    pub fn request_ref(&self) -> &Request {
        &self.req
    }

    pub fn request_mut_ref(&mut self) -> &mut Request {
        &mut self.req
    }

    pub fn storage_ref(&self) -> &AuthUserStorageProvider {
        &self.storage
    }

    pub fn storage_mut_ref(&mut self) -> &mut AuthUserStorageProvider {
        &mut self.storage
    }

    pub fn set_response(&mut self, resp: Response) {
        self.resp = Some(resp);
    }

    pub fn set_user(&mut self, user: Result<Option<AuthUser>, anyhow::Error>) {
        self.user = Some(user);
    }

    pub async fn register<F, Fut>(name: &str, callback: F)
    where
        F: Clone + Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Self> + Send + 'static,
    {
        let resolvers = Self::get_middleware().await;
        let arc_name = Arc::new(name.to_string());

        resolvers
            .next(move |mut resolver, next| {
                let cb = callback.clone();
                let name = arc_name.clone();
                Box::pin(async move {
                    if resolver.name_str_ref() == name.as_str() {
                        resolver = (cb)(resolver).await;
                        if resolver.user.is_some() || resolver.resp.is_some() {
                            return resolver;
                        }
                    }
                    next.call(resolver).await
                })
            })
            .await;
    }

    pub(crate) async fn guard(self) -> Self {
        Self::get_middleware().await.send(self).await
    }

    fn name_str_ref(&self) -> &str {
        self.name.as_str()
    }

    async fn get_middleware() -> Arc<simple_middleware::Manager<Self, Self>> {
        if let Some(r) = busybody::helpers::service_container().get().await {
            r
        } else {
            busybody::helpers::service_container()
                .set(simple_middleware::Manager::<Self, Self>::new())
                .await
                .get()
                .await
                .unwrap() // should never failed as we just registered the intance
        }
    }
}
