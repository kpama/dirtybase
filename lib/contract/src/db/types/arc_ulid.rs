use std::ops::Deref;
use std::{fmt::Debug, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::db::{base::helper::generate_arc_ulid, field_values::FieldValue};

use super::UlidField;

#[derive(Clone, Serialize, Deserialize)]
pub struct ArcUlidField(pub(crate) Arc<str>);

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
    type Target = Arc<str>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToString for ArcUlidField {
    fn to_string(&self) -> String {
        self.0.as_ref().to_string()
    }
}

impl Debug for ArcUlidField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<FieldValue> for ArcUlidField {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(v) => Self(v.clone().into()),
            _ => Self("".into()),
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
            _ => Some(ArcUlidField("".into())),
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
