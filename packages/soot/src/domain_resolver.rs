use std::sync::Arc;

use dirtybase_contract::{app_contract::Context, busybody};

use crate::domain::Domain;

#[derive(Clone)]
pub struct SootDomainResolver {
    context: Context,
    domains: Vec<Domain>,
}

impl SootDomainResolver {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            domains: Vec::new(),
        }
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub async fn register<F, Fut>(callback: F)
    where
        F: Clone + Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Domain> + Send + 'static,
    {
        let resolver = Self::get_middleware().await;
        resolver
            .next(move |mut resolver, next| {
                let cb = callback.clone();
                async move {
                    resolver.domains.push((cb)().await);
                    next.call(resolver).await
                }
            })
            .await;
    }

    pub async fn resolve(self) -> Vec<Domain> {
        Self::get_middleware().await.send(self).await.domains
    }

    async fn get_middleware() -> Arc<simple_middleware::Manager<Self, Self>> {
        if let Some(r) = busybody::helpers::service_container().get().await {
            r
        } else {
            let manager = simple_middleware::Manager::<Self, Self>::last(
                |resolver, _| async move { resolver },
            )
            .await;

            busybody::helpers::service_container()
                .set(manager)
                .await
                .get()
                .await
                .expect("could not resolve soot domain resolver")
        }
    }
}
