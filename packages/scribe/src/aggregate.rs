mod aggregate_wrapper;

use std::sync::{Arc, LockResult, Mutex, MutexGuard};

pub use aggregate_wrapper::*;
use dirtybase_contract::db::types::ArcUlidField;

use crate::{DispatchedDomainEvent, DomainEvent};

#[derive(Debug, Default)]
pub struct Aggregate {
    id: ArcUlidField,
    sequency: i64,
    events: Arc<Mutex<Vec<DispatchedDomainEvent>>>,
}

impl Aggregate {
    pub(crate) fn new<I: Into<ArcUlidField>>(id: I) -> Self {
        Self {
            id: id.into(),
            ..Default::default()
        }
    }

    pub fn record_event(&mut self, event: &impl DomainEvent) {
        let mut d = DispatchedDomainEvent::from(event);
        self.sequency += 1;
        d.sequence_number = self.sequency;
        if let Ok(mut lock) = self.events.lock() {
            lock.push(d)
        }
    }

    pub(crate) fn take_events(&self) -> LockResult<MutexGuard<Vec<DispatchedDomainEvent>>> {
        self.events.lock()
    }
}
