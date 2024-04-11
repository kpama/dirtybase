use chrono::{DateTime, Utc};

use super::FieldValue;

impl From<DateTime<Utc>> for FieldValue {
    fn from(value: DateTime<Utc>) -> Self {
        Self::DateTime(value)
    }
}

impl From<f32> for FieldValue {
    fn from(value: f32) -> Self {
        Self::F64(value as f64)
    }
}

impl From<f64> for FieldValue {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl From<i32> for FieldValue {
    fn from(value: i32) -> Self {
        Self::I64(value.into())
    }
}

impl From<i64> for FieldValue {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<&str> for FieldValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for FieldValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&String> for FieldValue {
    fn from(value: &String) -> Self {
        Self::String(value.to_owned())
    }
}

impl<'a> From<Option<&'a [String]>> for FieldValue {
    fn from(value: Option<&'a [String]>) -> Self {
        if let Some(v) = value {
            Self::Array(v.iter().map(|x| x.into()).collect())
        } else {
            Self::NotSet
        }
    }
}

impl From<u32> for FieldValue {
    fn from(value: u32) -> Self {
        Self::U64(value.into())
    }
}

impl From<u64> for FieldValue {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}
