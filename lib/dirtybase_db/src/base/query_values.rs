use super::{field_values::FieldValue, query::QueryBuilder};

#[derive(Debug)]
pub enum QueryValue {
    Field(FieldValue),
    SubQuery(QueryBuilder),
}

impl From<FieldValue> for QueryValue {
    fn from(value: FieldValue) -> Self {
        Self::Field(value)
    }
}
