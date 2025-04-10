use std::collections::HashMap;

use busstop::async_trait;
use dirtybase_contract::{
    app_contract::Context,
    config_contract::{ConfigResult, DirtyConfig, TryFromDirtyConfig},
};
use english_to_cron::str_cron_syntax;

use crate::JobId;

#[derive(Debug, Default, serde::Deserialize, Clone)]
pub struct CronConfig {
    #[serde(default)]
    enable: bool,
    #[serde(default)]
    jobs: HashMap<String, JobConfig>,
}

impl CronConfig {
    pub fn enable(&self) -> bool {
        self.enable
    }

    pub fn set_enable(&mut self, enable: bool) -> &mut Self {
        self.enable = enable;
        self
    }

    pub fn set_jobs(&mut self, jobs: HashMap<String, JobConfig>) -> &mut Self {
        self.jobs = jobs;
        self
    }

    pub fn jobs(&self) -> &HashMap<String, JobConfig> {
        &self.jobs
    }

    pub fn add_job(&mut self, name: &str, config: JobConfig) -> &mut Self {
        self.jobs.insert(name.to_string(), config);
        self
    }
}

#[async_trait]
impl TryFromDirtyConfig for CronConfig {
    type Returns = Self;
    async fn from_config(config: &DirtyConfig, _ctx: &Context) -> ConfigResult<Self::Returns> {
        Ok(config
            .optional_file("cron.toml", Some("DTY_CRON"))
            .build()
            .await
            .expect("could not create cron configuration")
            .try_deserialize::<Self>()?)
    }
}

#[derive(Debug, Default, serde::Deserialize, Clone)]
pub struct JobConfig {
    enable: bool,
    id: JobId,
    schedule: String,
    description: Option<String>,
}

impl JobConfig {
    pub fn new<T: ToString>(
        id: impl Into<JobId>,
        schedule: T,
        enable: bool,
        description: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            schedule: schedule.to_string(),
            enable,
            description,
        }
    }

    pub fn is_enable(&self) -> bool {
        self.enable
    }

    pub fn set_enable(&mut self, enable: bool) -> &mut Self {
        self.enable = enable;
        self
    }

    pub fn id(&self) -> &JobId {
        &self.id
    }

    pub fn schedule(&self) -> &str {
        &self.schedule
    }

    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    pub fn set_schedule<T: ToString>(&mut self, schedule: T) -> &mut Self {
        self.schedule =
            str_cron_syntax(&schedule.to_string()).unwrap_or_else(|_| schedule.to_string());
        self
    }
}
