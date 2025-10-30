use std::{collections::HashMap, sync::Arc};

use dirtybase_contract::prelude::Context;
use tokio::sync::RwLock;

use crate::{CronJobRegisterer, JobContext, JobId, config::CronConfig};

#[derive(Clone)]
pub struct CronJobManager {
    contexts: Arc<RwLock<HashMap<JobId, JobContext>>>,
}

impl Default for CronJobManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CronJobManager {
    pub fn new() -> Self {
        Self {
            contexts: Default::default(),
        }
    }

    pub async fn run(&self, cron_config: CronConfig, context: Context) {
        if !cron_config.enable() {
            return;
        }

        self.end().await;

        let mut w_lock = self.contexts.write().await;
        w_lock.drain();

        for config in cron_config.jobs() {
            if !config.is_enable() {
                continue;
            }

            if let Ok(wrapper) = CronJobRegisterer::new(context.clone(), config.clone())
                .get_handler()
                .await
            {
                match wrapper.schedule().await {
                    Ok(context) => {
                        w_lock.insert(config.id().clone(), context);
                    }
                    Err(e) => {
                        tracing::error!("could not start cron job: {:?}", e);
                    }
                }
            }
        }
    }

    pub async fn stop(&self) {
        let r_lock = self.contexts.read().await;
        for (_, ctx) in r_lock.iter() {
            _ = ctx.send(crate::event::CronJobCommand::Stop).await;
        }
    }

    pub async fn end(&self) {
        let mut w_lock = self.contexts.write().await;
        for (_, ctx) in w_lock.iter() {
            _ = ctx.send(crate::event::CronJobCommand::Exit).await;
        }
        w_lock.drain();
    }
}
