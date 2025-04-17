use std::sync::Arc;

use anyhow::anyhow;
use dirtybase_contract::prelude::Context;
use futures::future::BoxFuture;

use crate::{JobContext, JobId, config::CronConfig};

pub type JobHandler =
    Box<dyn FnMut(Arc<JobContext>) -> BoxFuture<'static, ()> + Send + Sync + 'static>;

pub struct CronJobRegisterer {
    context: Context,
    config: CronConfig,
}

impl CronJobRegisterer {
    pub fn new(context: Context, config: CronConfig) -> Self {
        Self { config, context }
    }
    pub fn context_ref(&self) -> &Context {
        &self.context
    }

    pub fn context(&self) -> Context {
        self.context.clone()
    }

    pub fn config_ref(&self) -> &CronConfig {
        &self.config
    }

    pub fn config(&self) -> CronConfig {
        self.config.clone()
    }

    pub async fn get_handler(self, job_id: JobId) -> Result<JobHandler, anyhow::Error> {
        Self::get_middleware().await.send((self, job_id)).await
    }

    pub async fn register<F, Fut>(job_id: JobId, callback: F)
    where
        F: Clone + Fn(&Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<JobHandler, anyhow::Error>> + Send + 'static,
    {
        let middleware = Self::get_middleware().await;

        middleware
            .next(move |params, next| {
                let cb = callback.clone();
                let id = job_id.clone();

                Box::pin(async move {
                    if params.1 == id {
                        return (cb)(&params.0).await;
                    }

                    next.call(params).await
                })
            })
            .await;
    }

    async fn get_middleware()
    -> Arc<simple_middleware::Manager<(Self, JobId), Result<JobHandler, anyhow::Error>>> {
        if let Some(r) = busybody::helpers::service_container().get().await {
            r
        } else {
            let manager = simple_middleware::Manager::<
                (Self, JobId),
                Result<JobHandler, anyhow::Error>,
            >::last(|(_, job_id), _| {
                Box::pin(async move { Err(anyhow!("No handler for cron job: {}", &job_id)) })
            })
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
