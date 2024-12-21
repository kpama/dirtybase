use std::sync::Arc;

use orsomafo::Dispatchable;

use crate::event::CronJobState;

#[derive(Clone)]
pub struct JobContext {
    id: String,
    container: Arc<busybody::ServiceContainer>,
}

impl JobContext {
    pub(crate) fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            container: Arc::new(busybody::ServiceContainer::proxy()),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn service_container(&self) -> &Arc<busybody::ServiceContainer> {
        &self.container
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
