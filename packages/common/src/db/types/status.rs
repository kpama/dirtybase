use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::db::field_values::FieldValue;

#[derive(Debug, Default, PartialEq, Hash, Eq, Clone, Serialize, Deserialize)]
pub enum StatusField {
    #[default]
    #[serde(alias = "active")]
    Active,
    #[serde(alias = "disabled")]
    Disabled,
    #[serde(alias = "pending")]
    Pending,
}

impl From<String> for StatusField {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "active" => Self::Active,
            "disabled" => Self::Disabled,
            "pending" => Self::Pending,
            _ => Self::Pending,
        }
    }
}

impl From<&str> for StatusField {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl FromStr for StatusField {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}

impl Display for StatusField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::Active => "active",
                Self::Disabled => "disabled",
                Self::Pending => "pending",
            }
        )
    }
}

impl From<FieldValue> for StatusField {
    fn from(value: FieldValue) -> Self {
        if let FieldValue::String(inner) = value {
            inner.into()
        } else {
            Self::Pending
        }
    }
}

impl From<StatusField> for FieldValue {
    fn from(value: StatusField) -> Self {
        Self::String(value.to_string())
    }
}
