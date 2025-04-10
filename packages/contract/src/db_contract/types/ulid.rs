use std::fmt::{Debug, Display};
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::db_contract::{base::helper::generate_ulid, field_values::FieldValue};

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

impl AsRef<str> for UlidField {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Debug for UlidField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for UlidField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
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
            _ => None,
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

impl From<&str> for UlidField {
    fn from(value: &str) -> Self {
        let x = Ulid::from_string(value).unwrap();
        UlidField(x.to_string().to_ascii_lowercase())
    }
}
