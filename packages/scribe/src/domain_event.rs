pub trait DomainEvent {
    fn event_type(&self) -> impl ToString;
    fn version(&self) -> impl ToString;
    fn data(&self) -> impl serde::Serialize
    where
        Self: Sized;
    fn metadata(&self) -> impl ToString {
        ""
    }
}

#[derive(Debug, Clone)]
pub struct DispatchedDomainEvent {
    event_type: String,
    event_version: String,
    pub(crate) sequence_number: i64,
    event_data: String,
    metadata: String,
    occurred_on: chrono::DateTime<chrono::Utc>,
}

impl<E> From<&E> for DispatchedDomainEvent
where
    E: DomainEvent,
{
    fn from(value: &E) -> Self {
        Self {
            event_type: value.event_type().to_string(),
            sequence_number: 0,
            event_version: value.version().to_string(),
            event_data: serde_json::to_string(&value.data()).unwrap(),
            metadata: value.metadata().to_string(),
            occurred_on: chrono::Utc::now(),
        }
    }
}
