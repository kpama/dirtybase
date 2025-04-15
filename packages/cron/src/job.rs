use std::{str::FromStr, sync::Arc};

use anyhow::anyhow;
use busstop::DispatchableCommand;
use chrono::Utc;
use cron::Schedule;
use english_to_cron::str_cron_syntax;
use futures::future::BoxFuture;
use orsomafo::Dispatchable;
use tokio::time::Instant;

use crate::{JobContext, JobId, event::CronJobState};

type JobHandler = Box<dyn FnMut(Arc<JobContext>) -> BoxFuture<'static, ()> + Send + Sync>;

pub struct CronJob {
    scheduler: cron::Schedule,
    handler: JobHandler,
    context: Arc<JobContext>,
}

impl CronJob {
    pub fn new(
        id: JobId,
        schedule: &str,
        handler: impl FnMut(Arc<JobContext>) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    ) -> Result<Self, anyhow::Error> {
        match Schedule::from_str(schedule) {
            Ok(scheduler) => {
                tracing::debug!("job '{}' scheduled to run '{}'", id, schedule);
                Ok(Self {
                    scheduler,
                    handler: Box::new(handler),
                    context: Arc::new(JobContext::new(id)),
                })
            }
            _ => {
                let s = str_cron_syntax(schedule);
                if s.is_err() {
                    return Err(anyhow!(s.unwrap_err()));
                }
                match Schedule::from_str(s.as_ref().unwrap()) {
                    Ok(s) => {
                        tracing::debug!(
                            "job '{}' scheduled to run '{}', original: '{}'",
                            id,
                            s.to_string(),
                            schedule
                        );
                        Ok(Self {
                            scheduler: s,
                            handler: Box::new(handler),
                            context: Arc::new(JobContext::new(id)),
                        })
                    }
                    Err(e) => Err(anyhow!(e)),
                }
            }
        }
    }

    pub async fn register(
        schedule: &str,
        handler: impl FnMut(Arc<JobContext>) -> BoxFuture<'static, ()> + Send + Sync + 'static,
        id: JobId,
    ) -> Result<Arc<JobContext>, anyhow::Error> {
        let job = Self::new(id, schedule, handler)?;
        let context = job.context();
        job.dispatch_command().await;
        Ok(context)
    }

    pub fn context(&self) -> Arc<JobContext> {
        self.context.clone()
    }

    pub(crate) async fn spawn(mut self) {
        tokio::spawn(async move { self.run().await });
    }

    async fn run(&mut self) {
        loop {
            for next in self.scheduler.upcoming(Utc).take(1) {
                let until = next - Utc::now();
                tokio::time::sleep_until(Instant::now() + until.to_std().unwrap()).await;
                CronJobState::Running {
                    id: self.context.id(),
                }
                .dispatch_event();
                tokio::task::block_in_place(|| async {
                    (self.handler)(self.context.clone()).await;
                    self.context.done().await;
                })
                .await;
            }
        }
    }
}

#[busstop::async_trait]
impl busstop::DispatchableCommand for CronJob {}
