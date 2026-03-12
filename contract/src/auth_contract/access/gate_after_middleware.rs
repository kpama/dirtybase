#![allow(unused)]

use std::{future::Future, sync::Arc};

use busybody::ServiceContainer;

use crate::{
    auth_contract::{Actor, GateAbility},
    prelude::Context,
};

use super::GateResponse;

#[derive(Debug, Clone)]
pub(crate) struct GateAfterMiddleware {
    pub(crate) sc: ServiceContainer,
    ability: GateAbility,
}

impl GateAfterMiddleware {
    pub(crate) fn new(sc: ServiceContainer, ability: GateAbility) -> Self {
        Self { sc, ability }
    }

    pub fn ability(&self) -> GateAbility {
        self.ability.clone()
    }
    pub fn ability_ref(&self) -> &GateAbility {
        &self.ability
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
            let manager = simple_middleware::Manager::<Self, Option<GateResponse>>::last(
                |resolver, _| async move {
                    //
                    let actor = if let Some(actor) = resolver.sc.get_type::<Actor>().await {
                        actor
                    } else {
                        return Some(GateResponse::deny());
                    };

                    let context = if let Some(ctx) = resolver.sc.get_type::<Context>().await {
                        ctx
                    } else {
                        return Some(GateResponse::deny());
                    };

                    tracing::info!(
                        "gate handled by last middlewere. ability: {}",
                        resolver.ability_ref().name(),
                    );
                    Some(
                        actor
                            .can(resolver.ability_ref().name(), &context)
                            .await
                            .into(),
                    )
                },
            )
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
