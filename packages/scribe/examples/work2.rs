use dirtybase_scribe::{Aggregate, AggregateTrait, DomainEvent, Repository};

#[tokio::main]
async fn main() {
    let repo = Repository::new();
    let mut customer = CustomerAggregate::new("John", "Doe", "Smith")
        .await
        .unwrap();

    customer
        .update_basic_info(Some("James"), None, Some("Brown"))
        .await
        .unwrap();

    customer = repo.save(customer).await;
    // dbg!("customer: {:#?}", &customer);
    println!("customer json: {:#?}", serde_json::to_string(&customer));
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct CustomerAggregate {
    aggregate: Aggregate,
    first_name: String,
    middle_name: String,
    last_name: String,
}

#[async_trait::async_trait]
impl AggregateTrait for CustomerAggregate {
    async fn apply(mut self, event: dirtybase_scribe::DispatchedDomainEvent) -> Self {
        match event.event_type.as_str() {
            "customer_created" => {
                let data = serde_json::from_value::<CustomerEvent>(event.event_data);
                if let Ok(CustomerEvent::CustomerCreated {
                    first_name,
                    middle_name,
                    last_name,
                }) = data
                {
                    self.first_name = first_name;
                    self.middle_name = middle_name;
                    self.last_name = last_name;
                }
            }
            "basic_info_updated" => {
                let data = serde_json::from_value::<CustomerEvent>(event.event_data);
                if let Ok(CustomerEvent::BasicInfoUpdated {
                    first_name,
                    middle_name,
                    last_name,
                }) = data
                {
                    if let Some(first_name) = first_name {
                        self.first_name = first_name;
                    }
                    if let Some(middle_name) = middle_name {
                        self.middle_name = middle_name;
                    }
                    if let Some(last_name) = last_name {
                        self.last_name = last_name;
                    }
                }
            }
            _ => {
                panic!("Unknown event type: {}", event.event_type);
            }
        }
        self
    }

    fn aggregate(&self) -> &Aggregate {
        &self.aggregate
    }
}

impl CustomerAggregate {
    pub async fn new(first_name: &str, middle_name: &str, last_name: &str) -> Result<Self, String> {
        if first_name.is_empty() || last_name.is_empty() {
            return Err(String::from("first name and last name cannot be empty"));
        }

        let event = CustomerEvent::CustomerCreated {
            first_name: first_name.to_string(),
            middle_name: middle_name.to_string(),
            last_name: last_name.to_string(),
        };
        let mut instance = Self {
            aggregate: Aggregate::default(),
            first_name: String::default(),
            middle_name: String::default(),
            last_name: String::default(),
        };

        instance.aggregate.record_event(&event).await;

        Ok(instance)
    }

    pub async fn update_basic_info(
        &mut self,
        first_name: Option<&str>,
        middle_name: Option<&str>,
        last_name: Option<&str>,
    ) -> Result<(), String> {
        if first_name.is_none() && middle_name.is_none() && last_name.is_none() {
            return Err(String::from("at least one field must be provided"));
        }

        let event = CustomerEvent::BasicInfoUpdated {
            first_name: first_name.map(|s| s.to_string()),
            middle_name: middle_name.map(|s| s.to_string()),
            last_name: last_name.map(|s| s.to_string()),
        };

        self.aggregate.record_event(&event).await;

        Ok(())
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum CustomerEvent {
    CustomerCreated {
        first_name: String,
        middle_name: String,
        last_name: String,
    },
    BasicInfoUpdated {
        first_name: Option<String>,
        middle_name: Option<String>,
        last_name: Option<String>,
    },
}

impl DomainEvent for CustomerEvent {
    fn data(&self) -> impl serde::Serialize
    where
        Self: Sized,
    {
        self
    }

    fn event_type(&self) -> impl ToString {
        match self {
            Self::CustomerCreated { .. } => "customer_created",
            Self::BasicInfoUpdated { .. } => "basic_info_updated",
        }
    }

    fn version(&self) -> impl ToString {
        "0.1"
    }
}
