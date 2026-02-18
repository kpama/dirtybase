use std::{
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
    sync::Arc,
};

use dirtybase_helper::uuid::{Uuid, Uuid25};
use serde::{Deserialize, Serialize};

use crate::db::field_values::FieldValue;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct ArcUuid7(Arc<Uuid>);

impl ArcUuid7 {
    pub fn new(value: Uuid) -> Result<Self, String> {
        if value.get_version_num() != 7 {
            tracing::error!("uuid is not version 7: {value}");
            return Err(format!("uuid is not version 7: {value}"));
        }
        Ok(ArcUuid7(Arc::new(value)))
    }

    pub fn to_uuid25(&self) -> Uuid25 {
        Uuid25::parse_unwrap(&self.to_string())
    }
    pub fn to_uuid25_string(&self) -> String {
        self.to_uuid25().to_string()
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
        write!(f, "{}", &self.0)
    }
}

impl Debug for ArcUuid7 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl From<FieldValue> for ArcUuid7 {
    fn from(value: FieldValue) -> Self {
        field_value_to_arc_uuid7(value).expect("could not parse field value to UUID7")
    }
}

impl From<&FieldValue> for ArcUuid7 {
    fn from(value: &FieldValue) -> Self {
        value.clone().into()
    }
}

impl From<FieldValue> for Option<ArcUuid7> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Null | FieldValue::NotSet => None,
            _ => field_value_to_arc_uuid7(value).ok(),
        }
    }
}

impl From<Uuid25> for ArcUuid7 {
    fn from(value: Uuid25) -> Self {
        value.to_hyphenated().to_string().try_into().unwrap()
    }
}

impl From<ArcUuid7> for FieldValue {
    fn from(value: ArcUuid7) -> Self {
        FieldValue::Uuid(*value.0)
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
        (*value).into()
    }
}

impl From<&ArcUuid7> for ArcUuid7 {
    fn from(value: &ArcUuid7) -> Self {
        value.clone()
    }
}

impl TryFrom<&str> for ArcUuid7 {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match Uuid::parse_str(value) {
            Ok(u) => Self::new(u),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl TryFrom<String> for ArcUuid7 {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl TryFrom<&String> for ArcUuid7 {
    type Error = String;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        value.try_into()
    }
}

impl<'de> Deserialize<'de> for ArcUuid7 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Uuid::deserialize(deserializer)?;
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
        FieldValue::Uuid(uuid) => Ok(uuid.into()),
        FieldValue::Binary(bytes) => {
            if bytes.len() > 16 {
                if let Ok(st) = String::from_utf8(bytes) {
                    Ok(ArcUuid7(Arc::new(Uuid::from_str(&st).unwrap())))
                } else {
                    Err("string is not a valid uuid7".to_string())
                }
            } else {
                Err(format!(
                    "UUID7 total length is less than 16: {}",
                    bytes.len()
                ))
            }
        }
        FieldValue::String(v) => Ok(ArcUuid7(Arc::new(Uuid::parse_str(&v).unwrap()))),
        _ => {
            tracing::error!("could not parse field value to uuid7");
            Err("could not parse field value to uuid7".to_string())
        }
    }
}
