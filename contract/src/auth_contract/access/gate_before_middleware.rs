#![allow(unused)]

use std::{future::Future, sync::Arc};

use busybody::ServiceContainer;

use super::GateResponse;

#[derive(Debug, Clone)]
pub(crate) struct GateBeforeMiddleware {
    pub(crate) sc: ServiceContainer,
    ability: u64,
}

impl GateBeforeMiddleware {
    pub(crate) fn new(sc: ServiceContainer, ability_hash: u64) -> Self {
        Self {
            sc,
            ability: ability_hash,
        }
    }

    pub fn ability(&self) -> u64 {
        self.ability
    }

    pub async fn handle(self) -> Option<GateResponse> {
        Self::get_middleware().await.send(self).await
    }

    pub async fn register<F, Fut>(before: F)
    where
        F: Clone + Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<GateResponse>> + Send + 'static,
    {
        //
        let resolvers = Self::get_middleware().await;
        resolvers
            .next(move |resolver, next| {
                let cb = before.clone();
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
                Box::pin(async move { None })
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
