use std::{str::FromStr, sync::Arc};

use anyhow::anyhow;
use chrono::Utc;
use cron::Schedule;
use english_to_cron::str_cron_syntax;
use futures::future::BoxFuture;
use orsomafo::Dispatchable;
use tokio::time::Instant;

use crate::{
    JobContext, JobId,
    event::{CronJobCommand, CronJobState},
};

type JobHandler = Arc<Box<dyn Fn(JobContext) -> BoxFuture<'static, ()> + Send + Sync>>;

pub struct CronJob {
    scheduler: cron::Schedule,
    handler: JobHandler,
    context: JobContext,
    receiver: tokio::sync::mpsc::Receiver<CronJobCommand>,
}

impl CronJob {
    pub fn new(
        id: JobId,
        schedule: &str,
        handler: impl Fn(JobContext) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    ) -> Result<Self, anyhow::Error> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        match Schedule::from_str(schedule) {
            Ok(scheduler) => {
                tracing::debug!("job '{}' scheduled to run '{}'", id, schedule);
                Ok(Self {
                    scheduler,
                    handler: Arc::new(Box::new(handler)),
                    context: JobContext::new(id, tx),
                    receiver: rx,
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
                            handler: Arc::new(Box::new(handler)),
                            context: JobContext::new(id, tx),
                            receiver: rx,
                        })
                    }
                    Err(e) => Err(anyhow!(e)),
                }
            }
        }
    }

    pub async fn schedule(
        schedule: &str,
        handler: impl Fn(JobContext) -> BoxFuture<'static, ()> + Send + Sync + 'static,
        id: JobId,
    ) -> Result<JobContext, anyhow::Error> {
        let job = Self::new(id, schedule, handler)?;
        let context = job.context();
        job.spawn().await;
        Ok(context)
    }

    pub fn context(&self) -> JobContext {
        self.context.clone()
    }

    pub fn context_ref(&self) -> &JobContext {
        &self.context
    }

    async fn run(&mut self) {
        let mut run = true;
        loop {
            let recv = self.receiver.recv();
            let until = self.scheduler.upcoming(Utc).next().unwrap() - Utc::now();
            let next_run = tokio::time::sleep_until(Instant::now() + until.to_std().unwrap());

            tokio::select! {
                _ = next_run => {
                    if run {
                        CronJobState::Running {
                            id: self.context.id(),
                        }.dispatch_event();

                        tokio::task::block_in_place(|| async {
                            (self.handler)(self.context.clone()).await;
                            self.context.done().await;
                        }).await;
                    }
                },

               Some(cmd) = recv  => {
                    match cmd {
                        CronJobCommand::Run => {
                            run = true;
                            tracing::debug!("cron job cmd 'run': {}", self.context_ref().id());
                        },
                        CronJobCommand::Stop => {
                            run = false;
                            tracing::debug!("cron job cmd 'stop': {} ", self.context_ref().id());
                        },
                        CronJobCommand::Exit=> {
                            tracing::debug!("cron job cmd 'ext': {}", self.context_ref().id());
                            return;
                        },
                    }
                }
            }
        }
    }

    pub(crate) async fn spawn(mut self) {
        tokio::spawn(async move { self.run().await });
    }
}
