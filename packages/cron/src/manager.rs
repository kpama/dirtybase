use std::{collections::HashMap, fmt::Display, sync::Arc};

use anyhow::anyhow;
use futures::future::BoxFuture;

use crate::{CronJob, JobContext, config::CronConfig, event::CronJobCommand};

pub struct JobManager {
    jobs: HashMap<
        JobId,
        Box<dyn FnMut(Arc<JobContext>) -> BoxFuture<'static, ()> + Send + Sync + 'static>,
    >,
    contexts: HashMap<JobId, Arc<JobContext>>,
    cron_config: CronConfig,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct JobId(Arc<String>); // job id must be in the format "namespace::name"

impl JobId {
    pub fn new(id: &str) -> Self {
        if !id.contains("::") {
            panic!("Cron job ID must be in the format namespace::name");
        }

        Self(Arc::new(id.replace(" ", "")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn validate(inner: &str) -> Result<Self, anyhow::Error> {
        if !inner.contains("::") {
            return Err(anyhow!("Cron job ID must be in the format namespace::name"));
        }

        Ok(Self(Arc::new(inner.replace(" ", ""))))
    }
}

impl TryFrom<String> for JobId {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::validate(&value)
    }
}

impl TryFrom<&str> for JobId {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::validate(value)
    }
}

impl Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl JobManager {
    pub fn new(cron_config: CronConfig) -> Self {
        Self {
            jobs: Default::default(),
            contexts: Default::default(),
            cron_config,
        }
    }

    pub fn register(
        &mut self,
        id: JobId,
        worker: impl FnMut(Arc<JobContext>) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    ) {
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
