use chrono::{DateTime, Utc};

use super::field_values::FieldValue;
use std::collections::HashMap;

pub type ColumnAndValue = HashMap<String, FieldValue>;

pub type InternalIdField = Option<i64>; // works across databases
pub type SingedIntegerField = Option<i64>;
pub type UnsignedIntegerField = Option<u64>;
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

#[derive(Debug, serde::Serialize)]
pub struct StructuredColumnAndValue {
    #[serde(flatten)]
    fields: ColumnAndValue,
}

impl StructuredColumnAndValue {
    pub fn new(fields: ColumnAndValue) -> Self {
        Self { fields }
    }

    pub fn from_results(results: Vec<ColumnAndValue>) -> Vec<Self> {
        results.into_iter().map(Self::from_column_value).collect()
    }

    pub fn from_a_result(result: ColumnAndValue) -> Self {
        Self::from_column_value(result)
    }
}
impl Default for StructuredColumnAndValue {
    fn default() -> Self {
        Self::new(ColumnAndValue::new())
    }
}

impl FromColumnAndValue for StructuredColumnAndValue {
    fn from_column_value(column_and_value: ColumnAndValue) -> Self {
        let mut data = ColumnAndValue::new();

        for kv in column_and_value.into_iter() {
            let pieces = kv.0.split('.').collect::<Vec<&str>>();
            if pieces.len() == 2 {
                if !data.contains_key(pieces[0]) {
                    data.insert(
                        pieces[0].to_string(),
                        FieldValue::Object(ColumnAndValue::new()),
                    );
                }
                match data.get_mut(pieces[0]) {
                    Some(field) => match field {
                        FieldValue::Object(obj) => {
                            obj.insert(pieces[1].to_string(), kv.1);
                        }
                        _ => (),
                    },
                    _ => (),
                }
            } else {
                data.insert(kv.0, kv.1);
            }
        }

        Self::new(data)
    }
}
