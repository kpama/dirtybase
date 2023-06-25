use chrono::{DateTime, Utc};

use super::field_values::FieldValue;
use std::collections::HashMap;

pub type ColumnAndValue = HashMap<String, FieldValue>;

pub type InternalIdField = Option<i64>; // works across databases
pub type SingedInteger = Option<i64>;
pub type UnsingedInteger = Option<u64>;
pub type StringField = Option<String>;
pub type UlidField = Option<String>;
pub type DateTimeField = Option<DateTime<Utc>>;
pub type BooleanField = Option<bool>;

pub trait IntoColumnAndValue {
    fn into_column_value(self) -> ColumnAndValue;
}

pub trait FromColumnAndValue {
    fn from_column_value(column_and_value: ColumnAndValue) -> Self;
}
