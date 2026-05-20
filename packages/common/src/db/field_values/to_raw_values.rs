use std::collections::HashSet;

use super::FieldValue;

pub mod field_to_chrono_datetime;
pub mod field_value_to_bool;
pub mod field_value_to_f64;
pub mod field_value_to_i16;
pub mod field_value_to_i32;
pub mod field_value_to_i64;
pub mod field_value_to_i8;
pub mod field_value_to_serde_json;
pub mod field_value_to_string;
pub mod field_value_to_u32;
pub mod field_value_to_u64;
pub mod field_value_to_vec;

impl<T> From<FieldValue> for Vec<T>
where
    T: From<FieldValue>,
{
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(v) => {
                if let Ok(l) = serde_json::from_str::<Vec<FieldValue>>(&v) {
                    return l.into_iter().map(|f| f.into()).collect();
                }
                Vec::new()
            }
            FieldValue::Array(v) => v.into_iter().map(|f| f.into()).collect(),
            _ => Vec::new(),
        }
    }
}

impl<T> From<&FieldValue> for Vec<T>
where
    T: From<FieldValue>,
{
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::String(v) => {
                if let Ok(l) = serde_json::from_str::<Vec<FieldValue>>(v) {
                    return l.into_iter().map(|f| f.into()).collect();
                }
                Vec::new()
            }
            FieldValue::Array(v) => v.iter().map(|f| f.clone().into()).collect(),
            _ => Vec::new(),
        }
    }
}

impl<T> From<FieldValue> for HashSet<T>
where
    T: std::cmp::Eq + std::hash::Hash + From<FieldValue>,
{
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(v) => {
                if let Ok(l) = serde_json::from_str::<Vec<FieldValue>>(&v) {
                    return l.into_iter().map(|f| f.into()).collect();
                }
                HashSet::new()
            }
            FieldValue::Array(v) => v.into_iter().map(|v| v.into()).collect(),
            _ => HashSet::new(),
        }
    }
}

impl<T> From<&FieldValue> for HashSet<T>
where
    T: std::cmp::Eq + std::hash::Hash + From<FieldValue>,
{
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::String(v) => {
                if let Ok(l) = serde_json::from_str::<Vec<FieldValue>>(v) {
                    return l.into_iter().map(|f| f.into()).collect();
                }
                HashSet::new()
            }
            FieldValue::Array(v) => v.iter().map(|f| f.clone().into()).collect(),
            _ => HashSet::new(),
        }
    }
}
