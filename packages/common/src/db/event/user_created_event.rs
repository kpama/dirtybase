use crate::db::types::ArcUuid7;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UserCreatedEvent {
    id: ArcUuid7,
}

impl UserCreatedEvent {
    pub fn new(id: ArcUuid7) -> Self {
        Self { id }
    }

    pub fn id_ref(&self) -> &ArcUuid7 {
        &self.id
    }

    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }
}

impl orsomafo::Dispatchable for UserCreatedEvent {}
