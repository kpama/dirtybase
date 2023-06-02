use crate::base::field_values::FieldValue;
use chrono::{DateTime, Utc};

impl From<FieldValue> for DateTime<Utc> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::DateTime(v) => v,
            _ => Utc::now(),
        }
    }
}

impl From<&FieldValue> for DateTime<Utc> {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::DateTime(v) => v.clone(),
            _ => Utc::now(),
        }
    }
}

impl From<FieldValue> for Option<DateTime<Utc>> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::DateTime(v) => Some(v),
            _ => None,
        }
    }
}
impl From<&FieldValue> for Option<DateTime<Utc>> {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::DateTime(v) => Some(v.clone()),
            _ => None,
        }
    }
}
