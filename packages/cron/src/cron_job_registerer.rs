use std::sync::Arc;

use anyhow::anyhow;
use dirtybase_contract::prelude::Context;
use futures::future::BoxFuture;

use crate::{CronJob, JobContext, JobId, config::CronConfig};

pub struct JobHandlerWrapper {
    handler: Box<dyn FnMut(JobContext) -> BoxFuture<'static, ()> + Send + Sync + 'static>,
    id: Option<JobId>,
}

impl JobHandlerWrapper {
    pub fn new(
        handler: impl FnMut(JobContext) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    ) -> Self {
        Self {
            handler: Box::new(handler),
            id: None,
        }
    }

    pub fn inner(
        self,
    ) -> Box<dyn FnMut(JobContext) -> BoxFuture<'static, ()> + Send + Sync + 'static> {
        self.handler
    }

    pub async fn schedule(self, schedule: &str) -> Result<JobContext, anyhow::Error> {
        CronJob::register(schedule, self.handler, self.id.unwrap_or_default()).await
    }
}

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

    pub async fn get_handler(self, job_id: JobId) -> Result<JobHandlerWrapper, anyhow::Error> {
        Self::get_middleware().await.send((self, job_id)).await
    }

    pub async fn register<F>(job_id: JobId, callback: F)
    where
        F: Clone + Fn(Self) -> JobHandlerWrapper + Send + 'static,
    {
        let middleware = Self::get_middleware().await;

        middleware
            .next(move |params, next| {
                let cb = callback.clone();
                let id = job_id.clone();

                Box::pin(async move {
                    if params.1 == id {
                        let mut wrapper = (cb)(params.0);
                        wrapper.id = Some(id);
                        return Ok(wrapper);
                    }

                    next.call(params).await
                })
            })
            .await;
    }

    async fn get_middleware()
    -> Arc<simple_middleware::Manager<(Self, JobId), Result<JobHandlerWrapper, anyhow::Error>>>
    {
        if let Some(manager) = busybody::helpers::service_container().get().await {
            manager
        } else {
            let manager = simple_middleware::Manager::<
                (Self, JobId),
                Result<JobHandlerWrapper, anyhow::Error>,
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
