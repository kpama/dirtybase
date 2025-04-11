use dirtybase_contract::db_contract::types::ArcUuid7;

use crate::{Aggregate, AggregateTrait};

pub struct Repository {}

impl Default for Repository {
    fn default() -> Self {
        Self::new()
    }
}

impl Repository {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn find<I: Into<ArcUuid7>>(&self, id: I) -> Result<Aggregate, String> {
        let agg_id: ArcUuid7 = id.into();
        println!("fetch aggregate: {}", agg_id);
        let aggregate = Aggregate::new(agg_id);
        Ok(aggregate)
    }

    pub async fn new_aggregate(&self) -> Aggregate {
        Aggregate::default()
    }

    pub async fn save(&self, custom: &mut impl AggregateTrait) {
        for an_event in custom.aggregate().take_events().await {
            println!("applying: {:?}", &an_event);
            custom.apply(an_event).await;
            // TODO: Save the event
        }
    }
}
