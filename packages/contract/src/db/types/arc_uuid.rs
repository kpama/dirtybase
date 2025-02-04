use std::{
    fmt::{Debug, Display},
    ops::Deref,
    sync::Arc,
};

use dirtybase_helper::uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::db::field_values::FieldValue;

#[derive(Clone)]
pub struct ArcUuid7(Arc<Uuid>);

impl ArcUuid7 {
    pub fn new(value: Uuid) -> Result<Self, String> {
        if value.get_version_num() != 7 {
            tracing::error!("uuid is not version 7: {}", value.to_string());
            return Err(format!("uuid is not version 7: {}", value.to_string()));
        }
        Ok(ArcUuid7(Arc::new(value)))
    }
}

impl Default for ArcUuid7 {
    fn default() -> Self {
        Self::new(dirtybase_helper::uuid::uuid_v7()).unwrap()
    }
}

impl Deref for ArcUuid7 {
    type Target = Arc<Uuid>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ArcUuid7 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0.to_string())
    }
}

impl Debug for ArcUuid7 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0.to_string())
    }
}

impl From<FieldValue> for ArcUuid7 {
    fn from(value: FieldValue) -> Self {
        field_value_to_arc_uuid7(value).expect("could not parsed field value to UUID7")
    }
}

impl From<&FieldValue> for ArcUuid7 {
    fn from(value: &FieldValue) -> Self {
        value.clone().into()
    }
}

impl From<FieldValue> for Option<ArcUuid7> {
    fn from(value: FieldValue) -> Self {
        field_value_to_arc_uuid7(value).ok()
    }
}

impl From<ArcUuid7> for FieldValue {
    fn from(value: ArcUuid7) -> Self {
        FieldValue::Binary(
            value
                .0
                .as_bytes()
                .iter()
                .map(|v| v.clone())
                .collect::<Vec<u8>>(),
        )
    }
}

impl From<&ArcUuid7> for FieldValue {
    fn from(value: &ArcUuid7) -> Self {
        value.clone().into()
    }
}

impl From<Uuid> for ArcUuid7 {
    fn from(value: Uuid) -> Self {
        ArcUuid7::new(value).expect("uuid is not version 7")
    }
}

impl From<&Uuid> for ArcUuid7 {
    fn from(value: &Uuid) -> Self {
        value.clone().into()
    }
}

impl From<&ArcUuid7> for ArcUuid7 {
    fn from(value: &ArcUuid7) -> Self {
        value.clone()
    }
}

impl From<&str> for ArcUuid7 {
    fn from(value: &str) -> Self {
        Self::new(Uuid::parse_str(&value).expect("str is not a valid UUID7")).unwrap()
    }
}

impl<'de> Deserialize<'de> for ArcUuid7 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Uuid::deserialize(deserializer).expect("Require a value that can be stringify");
        Ok(value.into())
    }
}

impl Serialize for ArcUuid7 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

fn field_value_to_arc_uuid7(value: FieldValue) -> Result<ArcUuid7, String> {
    match value {
        FieldValue::Binary(bytes) => Ok(ArcUuid7(Arc::new(Uuid::from_slice(&bytes).unwrap()))),
        FieldValue::String(v) => Ok(ArcUuid7(Arc::new(Uuid::parse_str(&v).unwrap()))),
        _ => {
            tracing::error!("could not parsed field value to uuid7");
            Err("could not parsed field value to uuid7".to_string())
        }
    }
}
