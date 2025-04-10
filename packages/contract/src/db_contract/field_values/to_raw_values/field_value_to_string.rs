use std::sync::Arc;

use crate::db_contract::field_values::FieldValue;

impl From<FieldValue> for String {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(v) => v,
            _ => "".into(),
        }
    }
}

impl From<FieldValue> for Arc<str> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(v) => v.into(),
            _ => "".into(),
        }
    }
}

impl From<FieldValue> for Arc<String> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(v) => v.into(),
            _ => "".to_string().into(),
        }
    }
}

impl From<&FieldValue> for String {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::String(v) => v.clone(),
            _ => "".into(),
        }
    }
}

impl From<&FieldValue> for Arc<str> {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::String(v) => v.clone().into(),
            _ => "".into(),
        }
    }
}

impl From<&FieldValue> for Arc<String> {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::String(v) => v.clone().into(),
            _ => "".to_string().into(),
        }
    }
}

impl From<FieldValue> for Option<String> {
    fn from(value: FieldValue) -> Self {
        if FieldValue::NotSet == value {
            None
        } else {
            Some(value.into())
        }
    }
}

impl From<FieldValue> for Option<Arc<str>> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(v) => Some(v.into()),
            _ => None,
        }
    }
}

impl From<FieldValue> for Option<Arc<String>> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(v) => Some(v.into()),
            _ => None,
        }
    }
}
