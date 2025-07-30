use dirtybase_scribe::{Aggregate, AggregateTrait, DomainEvent, Repository};

#[tokio::main]
async fn main() {
    let repo = Repository::new();
    let mut order_aggregate = OrderAggregate {
        aggregate: repo.new_aggregate().await,
    };

    _ = order_aggregate.create("28/08/2024").await;
    _ = order_aggregate.create("28/08/2024").await;
    _ = order_aggregate.create("28/08/2024").await;
    _ = order_aggregate.delete("28/08/2024").await;

    _ = repo.save(&mut order_aggregate).await;
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "_event_name")]
enum OrderEvent {
    OrderCreated { date_created: String },
    OrderUpdated { date_updated: String },
    OrderDeleted { date_delete: String },
}

impl OrderEvent {
    pub fn name(&self) -> &str {
        match self {
            Self::OrderCreated { .. } => "order_created",
            Self::OrderUpdated { .. } => "order_updated",
            Self::OrderDeleted { .. } => "order_deleted",
        }
    }
}

impl DomainEvent for OrderEvent {
    fn data(&self) -> impl serde::Serialize
    where
        Self: Sized,
    {
        self
    }

    fn event_type(&self) -> impl ToString {
        self.name()
    }

    fn version(&self) -> impl ToString {
        "0.1"
    }
}

struct OrderAggregate {
    aggregate: Aggregate,
}

impl OrderAggregate {
    pub async fn create(&mut self, date: &str) -> Result<(), String> {
        if date.is_empty() {
            return Err(String::from("date cannot be empty"));
        }

        self.aggregate
            .record_event(&OrderEvent::OrderCreated {
                date_created: date.to_string(),
            })
            .await;

        Ok(())
    }

    pub async fn delete(&mut self, date: &str) -> Result<(), String> {
        if date.is_empty() {
            return Err("deleted date cannot be empty".to_string());
        }

        self.aggregate
            .record_event(&OrderEvent::OrderDeleted {
                date_delete: date.to_string(),
            })
            .await;

        Ok(())
    }
}

#[async_trait::async_trait]
impl AggregateTrait for OrderAggregate {
    fn aggregate(&self) -> &Aggregate {
        &self.aggregate
    }

    async fn apply(&mut self, event: dirtybase_scribe::DispatchedDomainEvent) {
        println!("applying: {event:#?}");
    }
}
