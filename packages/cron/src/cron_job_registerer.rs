use std::sync::Arc;

use anyhow::anyhow;
use dirtybase_contract::prelude::Context;
use futures::future::BoxFuture;

use crate::{CronJob, JobContext, JobId, config::JobConfig};

pub struct JobHandlerWrapper {
    handler: Box<dyn FnMut(JobContext) -> BoxFuture<'static, ()> + Send + Sync + 'static>,
    config: Option<JobConfig>,
    id: Option<JobId>,
}

impl JobHandlerWrapper {
    pub fn new(
        handler: impl FnMut(JobContext) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    ) -> Self {
        Self {
            handler: Box::new(handler),
            config: None,
            id: None,
        }
    }

    pub fn inner(
        self,
    ) -> Box<dyn FnMut(JobContext) -> BoxFuture<'static, ()> + Send + Sync + 'static> {
        self.handler
    }

    pub async fn schedule(self) -> Result<JobContext, anyhow::Error> {
        let id = self.id.unwrap_or_default();
        if let Some(config) = self.config {
            if config.is_enable() {
                tracing::debug!("job {} scheduled", &id);
                return CronJob::schedule(config.schedule(), self.handler, id).await;
            }
        } else {
            return Err(anyhow!("job config does not exit"));
        }

        Err(anyhow!("Job is not enabled"))
    }
}

pub struct CronJobRegisterer {
    context: Context,
    config: JobConfig,
}

impl CronJobRegisterer {
    pub fn new(context: Context, config: JobConfig) -> Self {
        Self { config, context }
    }
    pub fn context_ref(&self) -> &Context {
        &self.context
    }

    pub fn context(&self) -> Context {
        self.context.clone()
    }

    pub fn config_ref(&self) -> &JobConfig {
        &self.config
    }

    pub fn config(&self) -> JobConfig {
        self.config.clone()
    }

    pub async fn get_handler(self) -> Result<JobHandlerWrapper, anyhow::Error> {
        Self::get_middleware().await.send(self).await
    }

    pub async fn register<F>(job_id: impl Into<JobId>, callback: F)
    where
        F: Clone + Fn(Self) -> JobHandlerWrapper + Send + 'static,
    {
        let middleware = Self::get_middleware().await;
        let job_id = job_id.into();
        middleware
            .next(move |reg, next| {
                let cb = callback.clone();
                let id = job_id.clone();

                Box::pin(async move {
                    if *reg.config_ref().id_ref() == id {
                        let config = reg.config();
                        let mut wrapper = (cb)(reg);
                        wrapper.id = Some(id);
                        wrapper.config = Some(config);
                        return Ok(wrapper);
                    }

                    next.call(reg).await
                })
            })
            .await;
    }

    async fn get_middleware()
    -> Arc<simple_middleware::Manager<Self, Result<JobHandlerWrapper, anyhow::Error>>> {
        if let Some(manager) = busybody::helpers::service_container().get().await {
            manager
        } else {
            let manager = simple_middleware::Manager::<
                Self,
                Result<JobHandlerWrapper, anyhow::Error>,
            >::last(|reg, _| {
                Box::pin(async move {
                    Err(anyhow!(
                        "No handler for cron job: {}",
                        &reg.config_ref().id_ref()
                    ))
                })
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
