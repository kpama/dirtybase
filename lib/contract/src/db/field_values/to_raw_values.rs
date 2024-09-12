use super::FieldValue;

pub mod field_to_chrono_datetime;
pub mod field_value_to_bool;
pub mod field_value_to_f64;
pub mod field_value_to_i64;
pub mod field_value_to_serde_json;
pub mod field_value_to_string;
pub mod field_value_to_u64;

impl<T> From<FieldValue> for Vec<T>
where
    T: From<FieldValue>,
{
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Array(v) => v.iter().map(|f| f.clone().into()).collect(),
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
            FieldValue::Array(v) => v.iter().map(|f| f.clone().into()).collect(),
            _ => Vec::new(),
        }
    }
}
