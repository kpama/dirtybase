use crate::db_contract::{base::query::QueryBuilder, field_values::FieldValue};

#[derive(Debug, Clone)]
pub enum QueryValue {
    Field(FieldValue),
    SubQuery(Box<QueryBuilder>),
}

impl<T: Into<FieldValue>> From<T> for QueryValue {
    fn from(value: T) -> Self {
        Self::Field(value.into())
    }
}
