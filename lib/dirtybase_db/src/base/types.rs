use std::collections::HashMap;

use super::field_values::FieldValue;

pub type ColumnAndValue = HashMap<String, FieldValue>;

pub trait IntoColumnAndValue {
    fn into_column_value(self) -> ColumnAndValue;
}

pub trait FromColumnAndValue {
    fn from_column_value(column_and_value: ColumnAndValue) -> Self;
}
