use std::fmt::Debug;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::db::{base::helper::generate_ulid, field_values::FieldValue};

use super::ArcUlidField;

#[derive(Clone, Serialize, Deserialize)]
pub struct UlidField(pub(crate) String);

impl Default for UlidField {
    fn default() -> Self {
        Self::new()
    }
}

impl UlidField {
    pub fn new() -> Self {
        Self(generate_ulid())
    }
}

impl Deref for UlidField {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToString for UlidField {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Debug for UlidField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<FieldValue> for UlidField {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(v) => Self(v),
            _ => Self("".to_string()),
        }
    }
}

impl From<&FieldValue> for UlidField {
    fn from(value: &FieldValue) -> Self {
        value.clone().into()
    }
}

impl From<UlidField> for FieldValue {
    fn from(value: UlidField) -> Self {
        FieldValue::String(value.0)
    }
}

impl From<&UlidField> for FieldValue {
    fn from(value: &UlidField) -> Self {
        value.clone().into()
    }
}

impl From<FieldValue> for Option<UlidField> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(v) => Some(UlidField(v)),
            _ => Some(UlidField("".to_string())),
        }
    }
}

impl From<ArcUlidField> for UlidField {
    fn from(value: ArcUlidField) -> Self {
        Self(value.0.to_string())
    }
}

impl From<&ArcUlidField> for UlidField {
    fn from(value: &ArcUlidField) -> Self {
        Self(value.to_string())
    }
}

impl From<&UlidField> for UlidField {
    fn from(value: &UlidField) -> Self {
        value.clone()
    }
}
