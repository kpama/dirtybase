use dirtybase_scribe::{Aggregate, AggregateTrait, DomainEvent, Repository};

#[tokio::main]
async fn main() {
    let repo = Repository {};
    let mut order_aggregate = OrderAggregate {
        date_created: String::new(),
        aggregate: repo.new_aggregate().await,
    };

    _ = order_aggregate.create("28/08/2024");
    _ = order_aggregate.create("28/08/2024");
    _ = order_aggregate.create("28/08/2024");

    _ = repo.save(&mut order_aggregate).await;
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "_event_name")]
enum OrderEvent {
    OrderCreated { date_created: String },
    OrderUpdated { date_updated: String },
    OrderDeleted { date_delete: String },
}

impl DomainEvent for OrderEvent {
    fn data(&self) -> &impl serde::Serialize
    where
        Self: Sized,
    {
        self
    }

    fn event_type(&self) -> &str {
        "event here"
    }

    fn version(&self) -> &str {
        "0.1"
    }
}

struct OrderAggregate {
    date_created: String,
    aggregate: Aggregate,
}

impl OrderAggregate {
    pub fn create(&mut self, date: &str) -> Result<(), String> {
        if date.is_empty() {
            return Err(String::from("date cannot be empty"));
        }

        self.aggregate.record_event(&OrderEvent::OrderCreated {
            date_created: date.to_string(),
        });

        Ok(())
    }
}

impl AggregateTrait for OrderAggregate {
    fn aggregate(&self) -> &Aggregate {
        &self.aggregate
    }

    fn apply(&mut self, event: dirtybase_scribe::DispatchedDomainEvent) {
        println!("applying: {:#?}", event);
    }
}
