use super::{query_operators::Operator, query_values::QueryValue};

#[derive(Debug, Clone)]
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
