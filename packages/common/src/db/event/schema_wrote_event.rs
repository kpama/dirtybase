use crate::db::base::schema::DatabaseKind;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SchemeWroteEvent {
    kind: DatabaseKind,
    timestamp: i64,
}

impl SchemeWroteEvent {
    pub fn new(kind: DatabaseKind, timestamp: i64) -> Self {
        Self { kind, timestamp }
    }

    pub fn kind(&self) -> &DatabaseKind {
        &self.kind
    }

    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

impl orsomafo::Dispatchable for SchemeWroteEvent {}
