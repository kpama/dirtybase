use std::sync::Arc;

use orsomafo::Dispatchable;
use tokio::sync::mpsc::error::SendError;

use crate::{
    JobId,
    event::{CronJobCommand, CronJobState},
};

#[derive(Clone)]
pub struct JobContext {
    id: JobId,
    container: Arc<busybody::ServiceContainer>,
    sender: tokio::sync::mpsc::Sender<CronJobCommand>,
}

impl JobContext {
    pub(crate) fn new(id: JobId, sender: tokio::sync::mpsc::Sender<CronJobCommand>) -> Self {
        Self {
            id,
            container: Arc::new(busybody::ServiceContainer::proxy()),
            sender,
        }
    }

    pub fn id(&self) -> JobId {
        self.id.clone()
    }

    pub fn id_ref(&self) -> &JobId {
        &self.id
    }

    pub fn service_container(&self) -> Arc<busybody::ServiceContainer> {
        self.container.clone()
    }

    pub async fn send(&self, cmd: CronJobCommand) -> Result<(), SendError<CronJobCommand>> {
        self.sender.send(cmd).await
    }

    pub fn fail(&self, reason: &str) {
        CronJobState::Failed {
            id: self.id.clone(),
            reason: reason.to_string(),
        }
        .dispatch_event();
    }

    pub(crate) async fn done(&self) {
        CronJobState::Completed {
            id: self.id.clone(),
        }
        .dispatch_event();
    }
}
