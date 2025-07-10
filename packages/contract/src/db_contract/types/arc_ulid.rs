use std::fmt::Display;
use std::ops::Deref;
use std::{fmt::Debug, sync::Arc};

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::db_contract::{base::helper::generate_arc_ulid, field_values::FieldValue};

use super::UlidField;

#[derive(Clone, Hash)]
pub struct ArcUlidField(pub(crate) Arc<String>);

impl<'de> Deserialize<'de> for ArcUlidField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value =
            UlidField::deserialize(deserializer).expect("Require a value that can be stringify");
        Ok(value.into())
    }
}

impl Serialize for ArcUlidField {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.as_ref())
    }
}

impl Default for ArcUlidField {
    fn default() -> Self {
        Self::new()
    }
}

impl ArcUlidField {
    pub fn new() -> Self {
        Self(generate_arc_ulid())
    }
}

impl Deref for ArcUlidField {
    type Target = Arc<String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Arc<String>> for ArcUlidField {
    fn as_ref(&self) -> &Arc<String> {
        &self.0
    }
}

impl Debug for ArcUlidField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl Display for ArcUlidField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<FieldValue> for ArcUlidField {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(v) => Self(v.clone().into()),
            _ => Self(Arc::new(String::new())),
        }
    }
}

impl From<&FieldValue> for ArcUlidField {
    fn from(value: &FieldValue) -> Self {
        value.clone().into()
    }
}

impl From<ArcUlidField> for FieldValue {
    fn from(value: ArcUlidField) -> Self {
        FieldValue::String(value.0.to_string())
    }
}

impl From<&ArcUlidField> for FieldValue {
    fn from(value: &ArcUlidField) -> Self {
        value.clone().into()
    }
}

impl From<FieldValue> for Option<ArcUlidField> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(v) => Some(ArcUlidField(v.into())),
            _ => Some(ArcUlidField(Arc::new(String::new()))),
        }
    }
}

impl From<UlidField> for ArcUlidField {
    fn from(value: UlidField) -> Self {
        Self(value.0.into())
    }
}

impl From<&UlidField> for ArcUlidField {
    fn from(value: &UlidField) -> Self {
        Self(value.0.clone().into())
    }
}

impl From<&ArcUlidField> for ArcUlidField {
    fn from(value: &ArcUlidField) -> Self {
        value.clone()
    }
}

impl From<&str> for ArcUlidField {
    fn from(value: &str) -> Self {
        let x = Ulid::from_string(value).unwrap();
        ArcUlidField(Arc::new(x.to_string().to_ascii_lowercase()))
    }
}
