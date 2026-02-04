use std::{collections::HashMap, future::Future, sync::Arc};

use futures::future::BoxFuture;
use simple_middleware::Manager;

use crate::app_contract::Context;

// type RegistererFn = Box<
//     dyn FnMut(
//             simple_middleware::Manager<(String, clap::ArgMatches, Context), ()>,
//         ) -> simple_middleware::Manager<(String, clap::ArgMatches, Context), ()>
//         + Send
//         + Sync
//         + 'static,
// >;

type RegistererFn = Box<
    dyn Fn(String, clap::ArgMatches, Context) -> BoxFuture<'static, Result<(), anyhow::Error>>
        + Send
        + Sync
        + 'static,
>;

#[derive(Default)]
pub struct CliMiddlewareManager(HashMap<String, Arc<RegistererFn>>);

impl CliMiddlewareManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<T, R>(&mut self, name: &str, middleware: T) -> &mut Self
    where
        T: Fn(String, clap::ArgMatches, Context) -> R + Send + Sync + 'static,
        R: Future<Output = Result<(), anyhow::Error>> + Send + 'static,
    {
        self.0.insert(
            name.to_string(),
            Arc::new(Box::new(move |name, arg, ctx| {
                let f = middleware(name, arg, ctx);
                Box::pin(async { f.await })
            })),
        );
        self
    }

    pub(crate) async fn apply<I, H>(
        &mut self,
        mut handler: H,
        order: impl IntoIterator<Item = I>,
    ) -> Arc<Manager<(String, clap::ArgMatches, Context), Result<(), anyhow::Error>>>
    where
        I: Into<String>,
        H: FnMut(
                String,
                clap::ArgMatches,
                Context,
            ) -> BoxFuture<'static, Result<(), anyhow::Error>>
            + Send
            + Sync
            + 'static,
    {
        let manager = Self::get_middleware_manager().await;
        manager
            .next(move |(a, b, c), _| {
                let fut = (handler)(a, b, c);
                async move {
                    let result = fut.await;
                    if let Err(e) = &result {
                        tracing::error!("command error: {}", e);
                    }
                    result
                }
            })
            .await;

        for n in order.into_iter() {
            let key = n.into();
            if let Some(middleware) = self.0.get_mut(&key) {
                let inner = middleware.clone();
                manager
                    .next(move |(name, arg, ctx), next| {
                        let f = inner(name.clone(), arg.clone(), ctx.clone());
                        async move {
                            let result = f.await;
                            if result.is_err() {
                                return result;
                            }
                            next.call((name, arg, ctx)).await
                        }
                    })
                    .await;
            } else {
                log::error!("could not find cli middleware: {key}",);
            }
        }

        manager
    }

    async fn get_middleware_manager() -> Arc<
        simple_middleware::Manager<(String, clap::ArgMatches, Context), Result<(), anyhow::Error>>,
    > {
        return match busybody::helpers::get_service().await {
            Some(m) => m,
            None => {
                let manager = simple_middleware::Manager::<
                    (String, clap::ArgMatches, Context),
                    Result<(), anyhow::Error>,
                >::new();
                busybody::helpers::service_container()
                    .set(manager)
                    .await
                    .get()
                    .await
                    .unwrap() // NOTE: Should never fail since we just added the instance
            }
        };
    }
}
