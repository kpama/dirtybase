use crate::db::types::ColumnAndValue;

#[derive(Debug, Default, Clone)]
pub struct QueryResult {
    affected: u64,
    last_insert_id: i64,
    record: Option<ColumnAndValue>,
}

impl QueryResult {
    pub fn new(affected: u64, last_insert_id: i64) -> Self {
        Self {
            affected,
            last_insert_id,
            record: None,
        }
    }

    pub fn new_record(record: ColumnAndValue) -> Self {
        Self {
            affected: 0,
            last_insert_id: 0,
            record: Some(record),
        }
    }

    pub fn rows_affected(&self) -> u64 {
        self.affected
    }

    pub fn record(&self) -> Option<&ColumnAndValue> {
        self.record.as_ref()
    }

    pub fn last_insert_id(&self) -> i64 {
        self.last_insert_id
    }
}
