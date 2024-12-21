use crate::db::types::UlidField;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UserCreatedEvent {
    id: UlidField,
}

impl UserCreatedEvent {
    pub fn new(id: UlidField) -> Self {
        Self { id }
    }
    pub fn id(&self) -> UlidField {
        self.id.clone()
    }
}

impl orsomafo::Dispatchable for UserCreatedEvent {}
