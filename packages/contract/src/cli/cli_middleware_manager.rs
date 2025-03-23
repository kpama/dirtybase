use std::{collections::HashMap, future::Future, sync::Arc};

use futures::future::BoxFuture;
use simple_middleware::Manager;

use crate::app::Context;

type RegistererFn = Box<
    dyn FnMut(
            simple_middleware::Manager<(String, clap::ArgMatches, Context), ()>,
        ) -> simple_middleware::Manager<(String, clap::ArgMatches, Context), ()>
        + Send
        + Sync
        + 'static,
>;

pub struct Registerer {
    wrapper: simple_middleware::Manager<(String, clap::ArgMatches, Context), ()>,
    name: Arc<String>,
    params: Option<HashMap<String, String>>,
}

impl Registerer {
    pub async fn middleware<F, Fut>(self, mut handler: F) -> Self
    where
        F: FnMut(String, clap::ArgMatches, Context, Option<HashMap<String, String>>) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        self.wrapper
            .next(move |(a, b, c), next| {
                //
                let x = (handler)(a, b, c, None);
                Box::pin(x)
            })
            .await;
        self
    }
}

#[derive(Default)]
pub struct CliMiddlewareManager(HashMap<String, RegistererFn>);

impl CliMiddlewareManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<T>(&mut self, name: &str, registerer: T) -> &mut Self
    where
        T: FnMut(
                simple_middleware::Manager<(String, clap::ArgMatches, Context), ()>,
            )
                -> simple_middleware::Manager<(String, clap::ArgMatches, Context), ()>
            + Send
            + Sync
            + 'static,
    {
        self.0.insert(name.to_string(), Box::new(registerer));
        self
    }

    pub async fn apply<I, H>(
        &mut self,
        mut handler: H,
        order: impl IntoIterator<Item = I>,
    ) -> Manager<(String, clap::ArgMatches, Context), ()>
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
        let mut manager =
            simple_middleware::Manager::<(String, clap::ArgMatches, Context), ()>::last(
                move |(a, b, c), _| {
                    let result = (handler)(a, b, c);
                    Box::pin(async move {
                        result.await;
                    })
                },
            )
            .await;

        for n in order.into_iter() {
            let key = n.into();
            if let Some(middleware) = self.0.get_mut(&key) {
                manager = (middleware)(manager);
            } else {
                // FIXME: Add translation
                log::error!("could not find web middleware: {}", key);
            }
        }

        //TODO: add the last middleware
        //  Add core middlewares here

        manager
    }
}
