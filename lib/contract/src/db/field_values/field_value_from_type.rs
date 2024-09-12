use std::{collections::HashMap, sync::Arc};

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

impl<'a> From<Option<&'a [&str]>> for FieldValue {
    fn from(value: Option<&'a [&str]>) -> Self {
        if let Some(v) = value {
            Self::Array(v.iter().map(|x| String::from(*x).into()).collect())
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

impl From<Arc<str>> for FieldValue {
    fn from(value: Arc<str>) -> Self {
        Self::String(String::from(value.as_ref()))
    }
}

impl From<serde_json::Number> for FieldValue {
    fn from(value: serde_json::Number) -> Self {
        if value.is_f64() {
            Self::F64(value.as_f64().unwrap_or_default())
        } else {
            Self::I64(value.as_i64().unwrap_or_default())
        }
    }
}
impl From<&serde_json::Number> for FieldValue {
    fn from(value: &serde_json::Number) -> Self {
        Self::from(value.clone())
    }
}

impl From<serde_json::value::Map<String, serde_json::Value>> for FieldValue {
    fn from(value: serde_json::value::Map<String, serde_json::Value>) -> Self {
        let mut map = HashMap::new();
        for (k, v) in value {
            map.insert(k, Self::from(v));
        }
        Self::Object(map)
    }
}

impl From<&serde_json::value::Map<String, serde_json::Value>> for FieldValue {
    fn from(value: &serde_json::value::Map<String, serde_json::Value>) -> Self {
        Self::from(value.clone())
    }
}

impl From<serde_json::Value> for FieldValue {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(v) => Self::Boolean(v),
            serde_json::Value::String(s) => Self::String(s),
            serde_json::Value::Number(n) => n.into(),
            serde_json::Value::Array(a) => Self::Array(
                a.into_iter()
                    .map(|m| Self::from(m))
                    .collect::<Vec<FieldValue>>(),
            ),
            serde_json::Value::Object(o) => Self::from(o),
        }
    }
}

impl From<&serde_json::Value> for FieldValue {
    fn from(value: &serde_json::Value) -> Self {
        Self::from(value.clone())
    }
}
