use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum CronJobState {
    Running { id: String },
    Completed { id: String },
    Failed { id: String, reason: String },
}

#[busstop::async_trait]
impl orsomafo::Dispatchable for CronJobState {}
