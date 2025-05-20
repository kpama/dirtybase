use std::{future::Future, ops::Deref, sync::Arc};

use axum::{
    extract::{rejection::PathRejection, FromRequestParts, Path},
    http::{request::Parts, StatusCode},
};
use serde::de::DeserializeOwned;

use crate::prelude::Context;

use super::HttpContext;

#[derive(Clone)]
pub struct Bind<T>(pub T);

impl<T, S> FromRequestParts<S> for Bind<T>
where
    T: Clone + Send + Sync + 'static,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        //..
        let context = if let Some(context) = parts.extensions.get::<Context>().cloned() {
            context
        } else {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, String::new()));
        };

        if let Some(m) = ModelBindResolver::new(context).await.bind::<T>().await {
            Ok(m)
        } else {
            // should return 404
            tracing::error!("could not resolve bind {}", std::any::type_name::<T>());
            Err((StatusCode::NOT_FOUND, String::new()))
        }
    }
}

impl<T> Deref for Bind<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for Bind<T> {
    fn from(value: T) -> Self {
        Bind(value)
    }
}

impl<T: std::any::Any> Bind<T> {
    pub async fn resolver<F, Fut>(callback: F)
    where
        F: Clone + Fn(ModelBindResolver) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<Bind<T>>> + Send + 'static,
    {
        ModelBindResolver::register(std::any::type_name::<T>(), callback).await
    }
}

pub struct ModelBindResolver {
    pub(crate) context: Context,
    pub(crate) http_ctx: HttpContext,
}

impl ModelBindResolver {
    pub(crate) async fn new(context: Context) -> Self {
        Self {
            http_ctx: context.get::<HttpContext>().await.unwrap(),
            context,
        }
    }

    pub fn context(&self) -> Context {
        self.context.clone()
    }

    pub fn context_ref(&self) -> &Context {
        &self.context
    }

    pub fn http_context_ref(&self) -> &HttpContext {
        &self.http_ctx
    }

    pub async fn get_path<T>(&mut self) -> Result<Path<T>, PathRejection>
    where
        T: DeserializeOwned + Send,
    {
        self.http_ctx.get_path().await
    }

    pub(crate) async fn bind<T: 'static>(self) -> Option<Bind<T>> {
        Self::get_middleware()
            .await
            .send((self, std::any::type_name::<T>()))
            .await
    }

    pub(crate) async fn register<T: 'static, F, Fut>(name: &str, callback: F)
    where
        F: Clone + Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<Bind<T>>> + Send + 'static,
    {
        let resolvers = Self::get_middleware().await;
        let arc_name = Arc::new(name.to_string());

        resolvers
            .next(move |(resolver, bind_name), next| {
                let cb = callback.clone();
                let name = arc_name.clone();
                Box::pin(async move {
                    if bind_name == name.as_str() {
                        return (cb)(resolver).await;
                    }
                    next.call((resolver, bind_name)).await
                })
            })
            .await;
    }

    pub(crate) async fn get_middleware<T: 'static>(
    ) -> Arc<simple_middleware::Manager<(Self, &'static str), Option<Bind<T>>>> {
        if let Some(r) = busybody::helpers::service_container().get().await {
            r
        } else {
            let manager =
                simple_middleware::Manager::<(Self, &'static str), Option<Bind<T>>>::last(
                    |(_, _), _| Box::pin(async move { None }),
                )
                .await;
            busybody::helpers::service_container()
                .set(manager)
                .await
                .get()
                .await
                .unwrap() // should never failed as we just registered the instance
        }
    }
}
