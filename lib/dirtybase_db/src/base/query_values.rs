use super::query::QueryBuilder;
use dirtybase_db_types::field_values::FieldValue;

#[derive(Debug, Clone)]
pub enum QueryValue {
    Field(FieldValue),
    SubQuery(QueryBuilder),
}

impl From<FieldValue> for QueryValue {
    fn from(value: FieldValue) -> Self {
        Self::Field(value)
    }
}
