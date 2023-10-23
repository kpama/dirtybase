#[derive(Debug, Clone)]
pub struct UserCreatedEvent {
    id: String,
}

impl UserCreatedEvent {
    pub fn new(id: &str) -> Self {
        Self { id: id.into() }
    }
    pub fn id(&self) -> String {
        self.id.clone()
    }
}

impl orsomafo::Dispatchable for UserCreatedEvent {}
