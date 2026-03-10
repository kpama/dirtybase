use std::{
    fmt::{Debug, Display},
    future::Future,
    sync::Arc,
};

use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

use crate::{
    auth_contract::{
        Actor,
        observable::{AuthSucceeded, AuthUnSuccessful},
        storage2::PermStorageProvider,
    },
    prelude::{Context, Observable},
};

pub struct GuardResolver {
    headers: HeaderMap,
    context: Context,
    storage: PermStorageProvider,
}

impl GuardResolver {
    pub fn new(headers: HeaderMap, context: Context, storage: PermStorageProvider) -> Self {
        Self {
            headers,
            context,
            storage,
        }
    }

    pub fn headers_ref(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn storage_ref(&self) -> &PermStorageProvider {
        &self.storage
    }

    pub fn context(&self) -> Context {
        self.context.clone()
    }

    pub fn context_ref(&self) -> &Context {
        &self.context
    }

    pub async fn register<F, Fut>(name: &str, callback: F)
    where
        F: Clone + Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = GuardResponse> + Send + 'static,
    {
        let resolvers = Self::get_middleware().await;
        let arc_name = Arc::new(name.to_string());

        resolvers
            .next(move |(resolver, guard_name), next| {
                let cb = callback.clone();
                let name = arc_name.clone();
                async move {
                    if guard_name == name.as_str() {
                        let ctx = resolver.context();
                        let mut guard_response = (cb)(resolver).await;

                        guard_response = guard_response.notify(&ctx).await;
                        if guard_response.is_success() {
                            let user = guard_response.actor().unwrap();
                            ctx.set(AuthSucceeded::dispatch_response(user, &ctx).await)
                                .await;
                        } else {
                            guard_response = AuthUnSuccessful::new(guard_response)
                                .notify(&ctx)
                                .await
                                .take_response();
                        }

                        if !guard_response.is_success() && !guard_response.has_response() {
                            guard_response.set_response(
                                GuardResponse::unauthorized().response().unwrap(), // NOTE: unwrap is okay here
                            );
                        }

                        return guard_response;
                    }
                    next.call((resolver, guard_name)).await
                }
            })
            .await;
    }

    pub async fn guard(self, name: &str) -> GuardResponse {
        Self::get_middleware()
            .await
            .send((self, name.to_string()))
            .await
    }

    async fn get_middleware() -> Arc<simple_middleware::Manager<(Self, String), GuardResponse>> {
        if let Some(r) = busybody::helpers::service_container().get().await {
            r
        } else {
            let manager = simple_middleware::Manager::<(Self, String), GuardResponse>::last(
                |(_, _), _| async move { GuardResponse::forbid() },
            )
            .await;
            busybody::helpers::service_container()
                .set(manager)
                .await
                .get()
                .await
                .unwrap() // Should never failed as we just registered the instance
        }
    }
}

/// Holds the response from the auth guard
///
/// Instance of this type if observable
pub struct GuardResponse {
    success: bool,
    actor: Option<Actor>,
    resp: Option<Response>,
}

impl GuardResponse {
    pub fn success(actor: Actor) -> Self {
        Self {
            success: true,
            actor: Some(actor),
            resp: None,
        }
    }

    pub fn failed(resp: Response) -> Self {
        Self {
            success: false,
            actor: None,
            resp: Some(resp),
        }
    }

    pub fn forbid() -> Self {
        Self {
            success: false,
            actor: None,
            resp: Some((StatusCode::FORBIDDEN, ()).into_response()),
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            success: false,
            actor: None,
            resp: Some((StatusCode::UNAUTHORIZED, ()).into_response()),
        }
    }

    pub fn fail_resp(resp: Response) -> Self {
        Self {
            success: false,
            actor: None,
            resp: Some(resp),
        }
    }

    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn actor(&self) -> Option<Actor> {
        self.actor.clone()
    }

    pub fn actor_ref(&self) -> Option<&Actor> {
        self.actor.as_ref()
    }

    pub fn has_actor(&self) -> bool {
        self.actor.is_some()
    }

    pub fn set_actor(&mut self, actor: Actor) -> &mut Self {
        self.actor = Some(actor);
        self
    }

    pub fn set_response(&mut self, resp: Response) -> &mut Self {
        self.resp = Some(resp);
        self
    }
    pub fn response(self) -> Option<Response> {
        self.resp
    }

    pub fn has_response(&self) -> bool {
        self.resp.is_some()
    }
}

impl Display for GuardResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "guard resp - is success: {}", self.is_success())
    }
}

impl Debug for GuardResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[async_trait::async_trait]
impl Observable for GuardResponse {}
