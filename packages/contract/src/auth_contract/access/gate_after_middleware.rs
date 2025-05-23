#![allow(unused)]

use std::{future::Future, sync::Arc};

use busybody::ServiceContainer;

use super::GateResponse;

#[derive(Debug, Clone)]
pub(crate) struct GateAfterMiddleware {
    pub(crate) sc: ServiceContainer,
    ability: Arc<String>,
}

impl GateAfterMiddleware {
    pub(crate) fn new(sc: ServiceContainer, ability: &str) -> Self {
        Self {
            sc,
            ability: Arc::new(ability.to_string()),
        }
    }

    pub fn ability_ref(&self) -> &str {
        self.ability.as_str()
    }

    pub fn ability(&self) -> Arc<String> {
        self.ability.clone()
    }

    pub async fn handle(self) -> Option<GateResponse> {
        Self::get_middleware().await.send(self).await
    }

    pub async fn register<F, Fut>(after: F)
    where
        F: Clone + Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<GateResponse>> + Send + 'static,
    {
        //
        let resolvers = Self::get_middleware().await;
        resolvers
            .next(move |resolver, next| {
                let cb = after.clone();
                Box::pin(async move {
                    let result = (cb)(resolver.clone()).await;
                    if result.is_some() {
                        return result;
                    }
                    next.call(resolver).await
                })
            })
            .await;
    }

    async fn get_middleware() -> Arc<simple_middleware::Manager<Self, Option<GateResponse>>> {
        if let Some(r) = busybody::helpers::service_container().get().await {
            r
        } else {
            let manager = simple_middleware::Manager::<Self, Option<GateResponse>>::last(|_, _| {
                //
                Box::pin(async move { Some(GateResponse::deny()) })
            })
            .await;
            busybody::helpers::service_container()
                .set(manager)
                .await
                .get()
                .await
                .unwrap()
        }
    }
}
