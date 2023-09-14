use orsomafo::Dispatchable;

pub struct DirtybaseUserLoggedInEvent {
    id: String,
}

impl DirtybaseUserLoggedInEvent {
    pub fn new(id: &str) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }
}

impl Dispatchable for DirtybaseUserLoggedInEvent {}
