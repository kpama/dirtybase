use crate::db::base::schema::DatabaseKind;

#[derive(Debug, Clone)]
pub struct SchemeWroteEvent {
    kind: DatabaseKind,
}

impl SchemeWroteEvent {
    pub fn new(kind: DatabaseKind) -> Self {
        Self { kind }
    }

    pub fn kind(&self) -> &DatabaseKind {
        &self.kind
    }
}

impl orsomafo::Dispatchable for SchemeWroteEvent {}
