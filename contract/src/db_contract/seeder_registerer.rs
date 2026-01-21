use std::{future::Future, sync::Arc};

use crate::prelude::Context;

use super::base::manager::Manager;

pub struct SeederRegisterer {
    cmd_name: String,
    manager: Manager,
    context: Context,
}

impl SeederRegisterer {
    pub fn new(name: &str, manager: Manager, context: Context) -> Self {
        Self {
            cmd_name: name.to_string(),
            manager,
            context,
        }
    }

    pub async fn seed(self) {
        Self::get_middleware().await.send(self).await;
    }

    pub async fn register<F, Fut>(name: &str, callback: F)
    where
        F: Clone + Fn(Manager, Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let registerer = Self::get_middleware().await;
        let arc_name = Arc::new(name.to_string());
        registerer
            .next(move |reg, next| {
                let cb = callback.clone();
                let name = arc_name.clone();
                Box::pin(async move {
                    if *reg.cmd_name == *name.as_ref() || &reg.cmd_name == "all" {
                        (cb)(reg.manager.clone(), reg.context.clone()).await;
                        if &reg.cmd_name != "all" {
                            return;
                        }
                    }
                    next.call(reg).await
                })
            })
            .await;
    }

    async fn get_middleware() -> Arc<simple_middleware::Manager<Self, ()>> {
        if let Some(r) = busybody::helpers::service_container().get().await {
            r
        } else {
            let manager = simple_middleware::Manager::<Self, ()>::last(|_registerer, _| {
                Box::pin(async move {
                    //
                })
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
