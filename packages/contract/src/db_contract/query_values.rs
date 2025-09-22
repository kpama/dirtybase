use serde::{Deserialize, Serialize};

use crate::db_contract::{base::query::QueryBuilder, field_values::FieldValue};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryValue {
    Null,
    Field(FieldValue),
    ColumnName(String),
    SubQuery(Box<QueryBuilder>),
}

impl<T: Into<FieldValue>> From<T> for QueryValue {
    fn from(value: T) -> Self {
        Self::Field(value.into())
    }
}
