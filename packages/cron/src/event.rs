use serde::{Deserialize, Serialize};

use crate::JobId;

#[derive(Clone, Serialize, Deserialize)]
pub enum CronJobState {
    Running { id: JobId },
    Completed { id: JobId },
    Failed { id: JobId, reason: String },
}

#[derive(Clone, Serialize, Deserialize)]
pub enum CronJobCommand {
    Stop,
    Run,
    Exit,
}

#[orsomafo::async_trait]
impl orsomafo::Dispatchable for CronJobState {}

#[orsomafo::async_trait]
impl orsomafo::Dispatchable for CronJobCommand {}
