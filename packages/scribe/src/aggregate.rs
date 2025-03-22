mod aggregate_wrapper;

use std::sync::Arc;

pub use aggregate_wrapper::*;
use dirtybase_contract::db::types::ArcUuid7;
use tokio::sync::RwLock;

use crate::{DispatchedDomainEvent, DomainEvent};

#[derive(Debug, Default)]
pub struct Aggregate {
    id: ArcUuid7,
    sequency: i64,
    events: Arc<RwLock<Vec<DispatchedDomainEvent>>>,
}

impl Aggregate {
    pub(crate) fn new<I: Into<ArcUuid7>>(id: I) -> Self {
        Self {
            id: id.into(),
            ..Default::default()
        }
    }

    pub async fn record_event(&mut self, event: &impl DomainEvent) {
        let mut d = DispatchedDomainEvent::from(event);
        let mut w_lock = self.events.write().await;

        self.sequency += 1;
        d.sequence_number = self.sequency;
        w_lock.push(d)
    }

    pub(crate) async fn take_events(&self) -> Vec<DispatchedDomainEvent> {
        let mut w_lock = self.events.write().await;
        w_lock.drain(0..).collect()
    }
}
