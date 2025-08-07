use std::{
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
    sync::Arc,
};

use dirtybase_helper::uuid::{Uuid, Uuid25};
use serde::{Deserialize, Serialize};

use crate::db_contract::field_values::FieldValue;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct ArcUuid4(Arc<Uuid>);

impl ArcUuid4 {
    pub fn new(value: Uuid) -> Result<Self, String> {
        if value.get_version_num() != 4 {
            tracing::error!("uuid is not version 4: {value}");
            return Err(format!("uuid is not version 4: {value}"));
        }
        Ok(ArcUuid4(Arc::new(value)))
    }

    pub fn to_uuid25(&self) -> Uuid25 {
        Uuid25::parse_unwrap(&self.to_string())
    }
    pub fn to_uuid25_string(&self) -> String {
        self.to_uuid25().to_string()
    }
}

impl Default for ArcUuid4 {
    fn default() -> Self {
        Self::new(dirtybase_helper::uuid::uuid_v4()).unwrap()
    }
}

impl Deref for ArcUuid4 {
    type Target = Arc<Uuid>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ArcUuid4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0.to_string())
    }
}

impl Debug for ArcUuid4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0.to_string())
    }
}

impl From<FieldValue> for ArcUuid4 {
    fn from(value: FieldValue) -> Self {
        field_value_to_arc_uuid4(value).expect("could not parsed field value to UUID4")
    }
}

impl From<&FieldValue> for ArcUuid4 {
    fn from(value: &FieldValue) -> Self {
        value.clone().into()
    }
}

impl From<FieldValue> for Option<ArcUuid4> {
    fn from(value: FieldValue) -> Self {
        field_value_to_arc_uuid4(value).ok()
    }
}

impl From<Uuid25> for ArcUuid4 {
    fn from(value: Uuid25) -> Self {
        value.to_hyphenated().to_string().try_into().unwrap()
    }
}

impl From<ArcUuid4> for FieldValue {
    fn from(value: ArcUuid4) -> Self {
        FieldValue::Uuid(*value.0)
    }
}

impl From<&ArcUuid4> for FieldValue {
    fn from(value: &ArcUuid4) -> Self {
        value.clone().into()
    }
}

impl From<Uuid> for ArcUuid4 {
    fn from(value: Uuid) -> Self {
        ArcUuid4::new(value).expect("uuid is not version 4")
    }
}

impl From<&Uuid> for ArcUuid4 {
    fn from(value: &Uuid) -> Self {
        (*value).into()
    }
}

impl From<&ArcUuid4> for ArcUuid4 {
    fn from(value: &ArcUuid4) -> Self {
        value.clone()
    }
}

impl TryFrom<&str> for ArcUuid4 {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match Uuid::parse_str(value) {
            Ok(u) => Self::new(u),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl TryFrom<String> for ArcUuid4 {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl<'de> Deserialize<'de> for ArcUuid4 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Uuid::deserialize(deserializer)?;
        Ok(value.into())
    }
}

impl Serialize for ArcUuid4 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

fn field_value_to_arc_uuid4(value: FieldValue) -> Result<ArcUuid4, String> {
    match value {
        FieldValue::Uuid(uuid) => Ok(uuid.into()),
        FieldValue::Binary(bytes) => {
            if bytes.len() > 16 {
                if let Ok(st) = String::from_utf8(bytes) {
                    Ok(ArcUuid4(Arc::new(Uuid::from_str(&st).unwrap())))
                } else {
                    Err("string is not a valid uuid4".to_string())
                }
            } else {
                Ok(ArcUuid4(Arc::new(Uuid::from_slice(&bytes).unwrap())))
            }
        }
        FieldValue::String(v) => Ok(ArcUuid4(Arc::new(Uuid::parse_str(&v).unwrap()))),
        _ => {
            tracing::error!("could not parsed field value to uuid4");
            Err("could not parsed field value to uuid4".to_string())
        }
    }
}
