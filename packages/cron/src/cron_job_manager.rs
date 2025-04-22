use std::{collections::HashMap, sync::Arc};

use futures::future::BoxFuture;

use crate::{CronJob, JobContext, JobId, config::CronConfig};

pub struct CronJobManager {
    jobs: HashMap<
        JobId,
        Box<dyn FnMut(JobContext) -> BoxFuture<'static, ()> + Send + Sync + 'static>,
    >,
    contexts: HashMap<JobId, JobContext>,
    cron_config: CronConfig,
}

impl CronJobManager {
    pub fn new(cron_config: CronConfig) -> Self {
        Self {
            jobs: Default::default(),
            contexts: Default::default(),
            cron_config,
        }
    }

    pub fn register<W>(&mut self, id: JobId, worker: W)
    where
        W: FnMut(JobContext) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        self.jobs.insert(id, Box::new(worker));
    }

    pub async fn run(&mut self) {
        if !self.cron_config.enable() {
            return;
        }

        for config in self.cron_config.jobs().values() {
            if let Some(job) = self.jobs.remove(config.id_ref()) {
                match CronJob::register(config.schedule(), job, config.id()).await {
                    Ok(context) => {
                        self.contexts.insert(config.id().clone(), context);
                    }
                    Err(e) => {
                        tracing::error!("could not start cron job: {:?}", e);
                    }
                }
            }
        }
    }
}
