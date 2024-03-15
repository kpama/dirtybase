use orsomafo::Dispatchable;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct UserLoggedInEvent {
    id: String,
}

impl UserLoggedInEvent {
    pub fn new(id: &str) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }
}

impl Dispatchable for UserLoggedInEvent {}
