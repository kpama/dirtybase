use dirtybase_contract::db::types::ArcUlidField;

use crate::{Aggregate, AggregateTrait, DispatchedDomainEvent};

pub struct Repository {}

impl Repository {
    pub async fn find<I: Into<ArcUlidField>>(&self, id: I) -> Result<Aggregate, String> {
        let agg_id: ArcUlidField = id.into();
        println!("fetch aggregate: {}", agg_id);
        let aggregate = Aggregate::new(agg_id);
        Ok(aggregate)
    }

    pub async fn new_aggregate(&self) -> Aggregate {
        Aggregate::default()
    }

    pub async fn save(&self, custom: &mut impl AggregateTrait) {
        let mut events: Vec<DispatchedDomainEvent> = Vec::new();
        if let Ok(mut lock) = custom.aggregate().take_events() {
            events = lock.drain(0..).collect::<Vec<_>>();
            drop(lock);
        }

        for an_event in &events {
            // TODO: save the data

            // TODO: Dispatch events
        }

        for an_event in events {
            // TODO: apply changes
            custom.apply(an_event)
        }

        // println!("{:#?}", events);
    }
}
