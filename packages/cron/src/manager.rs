use std::{collections::HashMap, sync::Arc};

use futures::future::BoxFuture;

use crate::{config::CronConfig, CronJob, JobContext};

pub struct JobManager {
    jobs: HashMap<
        JobId,
        Box<dyn FnMut(Arc<JobContext>) -> BoxFuture<'static, ()> + Send + Sync + 'static>,
    >,
    contexts: HashMap<JobId, Arc<JobContext>>,
    cron_config: CronConfig,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, serde::Deserialize)]
pub struct JobId(String); // job id must be in the format "namespace::name"

impl JobId {
    pub fn new(id: &str) -> Self {
        if !id.contains("::") {
            panic!("Cron job ID must be in the format namespace::name");
        }

        Self(id.replace(" ", "").to_ascii_lowercase())
    }

    pub fn id(&self) -> &str {
        &self.0
    }
}

impl ToString for JobId {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl From<&str> for JobId {
    fn from(value: &str) -> Self {
        JobId::new(value)
    }
}

impl From<String> for JobId {
    fn from(value: String) -> Self {
        JobId::new(&value)
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
        id: &str,
        job: impl FnMut(Arc<JobContext>) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    ) {
        self.jobs.insert(id.into(), Box::new(job));
    }

    pub async fn run(&mut self) {
        if !self.cron_config.enable() {
            return;
        }

        for config in self.cron_config.jobs().values() {
            if let Some(job) = self.jobs.remove(config.id()) {
                match CronJob::register(config.schedule(), job, config.id().id()).await {
                    Ok(context) => {
                        self.contexts.insert(config.id().clone(), context);
                    }
                    Err(e) => {
                        panic!("could not start cron job: {:?}", e);
                    }
                }
            }
        }
    }
}
