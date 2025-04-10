use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use dirtybase_helper::snowflake::generate_snowflake_id;
use serde::{Deserialize, Serialize};

use crate::db_contract::field_values::FieldValue;

#[derive(Serialize, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SnowflakeField(pub(crate) u64);

impl Default for SnowflakeField {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for SnowflakeField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for SnowflakeField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SnowflakeField {
    pub fn new() -> Self {
        Self(generate_snowflake_id())
    }
}

impl Deref for SnowflakeField {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<FieldValue> for SnowflakeField {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::I64(v) => Self(v as u64),
            FieldValue::U64(v) => Self(v),
            _ => Self(0),
        }
    }
}

impl From<&FieldValue> for SnowflakeField {
    fn from(value: &FieldValue) -> Self {
        value.clone().into()
    }
}

impl From<SnowflakeField> for FieldValue {
    fn from(value: SnowflakeField) -> Self {
        FieldValue::U64(value.0)
    }
}

impl From<&SnowflakeField> for FieldValue {
    fn from(value: &SnowflakeField) -> Self {
        value.clone().into()
    }
}

impl From<FieldValue> for Option<SnowflakeField> {
    fn from(value: FieldValue) -> Self {
        let v = SnowflakeField::from(value);
        if v.0 > 0 {
            Some(v)
        } else {
            None
        }
    }
}

impl From<&SnowflakeField> for SnowflakeField {
    fn from(value: &SnowflakeField) -> Self {
        value.clone()
    }
}
