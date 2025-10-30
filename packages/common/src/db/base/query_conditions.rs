use serde::{Deserialize, Serialize};

use crate::db::query_values::QueryValue;

use super::query_operators::Operator;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub column: String,
    pub operator: Operator,
    pub value: QueryValue,
}

impl Condition {
    pub fn new<T: Into<QueryValue>, C: ToString>(column: C, operator: Operator, value: T) -> Self {
        Self {
            column: column.to_string(),
            operator,
            value: value.into(),
        }
    }

    pub fn column(&self) -> &String {
        &self.column
    }

    pub fn operator(&self) -> &Operator {
        &self.operator
    }

    pub fn value(&self) -> &QueryValue {
        &self.value
    }
}
