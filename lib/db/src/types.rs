use chrono::{DateTime, Utc};

use super::field_values::FieldValue;
use std::collections::HashMap;

pub type ColumnAndValue = HashMap<String, FieldValue>;

pub type InternalIdField = Option<i64>; // works across databases
pub type SingedIntegerField = Option<i64>;
pub type UnsignedIntegerField = Option<u64>;
pub type FloatField = Option<f64>;
pub type StringField = Option<String>;
pub type UlidField = Option<String>;
pub type DateTimeField = Option<DateTime<Utc>>;
pub type TimestampField = Option<DateTime<Utc>>;
pub type BooleanField = Option<bool>;

pub trait IntoColumnAndValue {
    fn into_column_value(self) -> ColumnAndValue;
}

pub trait FromColumnAndValue {
    fn from_column_value(column_and_value: ColumnAndValue) -> Self;
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct StructuredColumnAndValue {
    #[serde(flatten)]
    fields: ColumnAndValue,
}

impl StructuredColumnAndValue {
    pub fn new(fields: ColumnAndValue) -> Self {
        Self { fields }
    }

    pub fn fields(self) -> ColumnAndValue {
        self.fields
    }

    pub fn from_results(results: Vec<ColumnAndValue>) -> Vec<Self> {
        results.into_iter().map(Self::from_column_value).collect()
    }

    pub fn from_results_into<T: FromColumnAndValue>(results: Vec<ColumnAndValue>) -> Vec<T> {
        let structured_results = Self::from_results(results);
        let mut data = Vec::new();

        for structured in structured_results {
            for entry in structured.fields {
                if let FieldValue::Object(kv) = entry.1 {
                    data.push(T::from_column_value(kv));
                }
            }
        }

        data
    }

    pub fn from_a_result(result: ColumnAndValue) -> Self {
        Self::from_column_value(result)
    }

    pub fn get(&self, key: &str) -> Option<&FieldValue> {
        self.fields.get(key)
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
            data = build_structure(data, pieces, kv.1);
        }

        Self::new(data)
    }
}

fn build_structure(
    mut built: ColumnAndValue,
    mut pieces: Vec<&str>,
    value: FieldValue,
) -> ColumnAndValue {
    match pieces.len() {
        1 => {
            let name = pieces.remove(0);
            built.insert(name.to_string(), value);
        }
        _ => {
            let name = pieces.remove(0);
            match built.get_mut(name) {
                Some(FieldValue::Object(obj)) => {
                    *obj = build_structure(obj.clone(), pieces, value);
                }
                None => {
                    built.insert(
                        name.to_string(),
                        FieldValue::Object(build_structure(ColumnAndValue::new(), pieces, value)),
                    );
                }
                _ => {
                    dbg!("shouldn't get this far");
                }
            }
        }
    }

    built
}

// Allows the user to pass a hashmap if they want to
impl IntoColumnAndValue for ColumnAndValue {
    fn into_column_value(self) -> ColumnAndValue {
        self
    }
}
