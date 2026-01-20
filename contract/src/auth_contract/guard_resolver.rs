use std::{future::Future, sync::Arc};

use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

use crate::prelude::Context;

use super::{AuthUser, AuthUserStorageProvider};

pub struct GuardResolver {
    headers: HeaderMap,
    storage: AuthUserStorageProvider,
    context: Context,
}

impl GuardResolver {
    pub fn new(headers: HeaderMap, context: Context, storage: AuthUserStorageProvider) -> Self {
        Self {
            headers,
            context,
            storage,
        }
    }

    pub fn headers_ref(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn storage_ref(&self) -> &AuthUserStorageProvider {
        &self.storage
    }

    pub fn context(&self) -> Context {
        self.context.clone()
    }

    pub fn context_ref(&self) -> &Context {
        &self.context
    }

    pub fn storage_mut_ref(&mut self) -> &mut AuthUserStorageProvider {
        &mut self.storage
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
                        let result = (cb)(resolver).await;

                        if result.is_success() {
                            ctx.set(result.user().unwrap()).await;
                        }

                        return result;
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

pub struct GuardResponse {
    success: bool,
    user: Option<AuthUser>,
    resp: Option<Response>,
}

impl GuardResponse {
    pub fn success(user: AuthUser) -> Self {
        Self {
            success: true,
            user: Some(user),
            resp: None,
        }
    }

    pub fn failed(resp: Response) -> Self {
        Self {
            success: false,
            user: None,
            resp: Some(resp),
        }
    }

    pub fn forbid() -> Self {
        Self {
            success: false,
            user: None,
            resp: Some((StatusCode::FORBIDDEN, ()).into_response()),
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            success: false,
            user: None,
            resp: Some((StatusCode::FORBIDDEN, ()).into_response()),
        }
    }

    pub fn fail_resp(resp: Response) -> Self {
        Self {
            success: false,
            user: None,
            resp: Some(resp),
        }
    }

    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn user(&self) -> Option<AuthUser> {
        self.user.clone()
    }

    pub fn response(self) -> Option<Response> {
        self.resp
    }
}
