use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::db::field_values::FieldValue;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum AuthUserStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "inactive")]
    Inactive,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "suspended")]
    Suspended,
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "custom")]
    Custom(String),
}

pub const AUTH_USER_STATUS_ACTIVE: &str = "active";
pub const AUTH_USER_STATUS_INACTIVE: &str = "inactive";
pub const AUTH_USER_STATUS_PENDING: &str = "pending";
pub const AUTH_USER_STATUS_SUSPENDED: &str = "suspended";
pub const AUTH_USER_STATUS_UNKNOWN: &str = "unknown";

impl From<AuthUserStatus> for FieldValue {
    fn from(value: AuthUserStatus) -> Self {
        value.to_string().into()
    }
}

impl From<&AuthUserStatus> for FieldValue {
    fn from(value: &AuthUserStatus) -> Self {
        value.to_string().into()
    }
}

impl From<FieldValue> for AuthUserStatus {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(s) => match s.to_ascii_lowercase().as_str() {
                AUTH_USER_STATUS_ACTIVE => Self::Active,
                AUTH_USER_STATUS_INACTIVE => Self::Inactive,
                AUTH_USER_STATUS_SUSPENDED => Self::Suspended,
                AUTH_USER_STATUS_UNKNOWN => Self::Pending,
                _ => Self::Unknown,
            },
            _ => Self::Unknown,
        }
    }
}

impl From<&FieldValue> for AuthUserStatus {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::String(v) => FieldValue::from(v).into(),
            _ => AuthUserStatus::Unknown,
        }
    }
}

impl Display for AuthUserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthUserStatus::Pending => write!(f, "{}", AUTH_USER_STATUS_PENDING),
            AuthUserStatus::Active => write!(f, "{}", AUTH_USER_STATUS_ACTIVE),
            AuthUserStatus::Inactive => write!(f, "{}", AUTH_USER_STATUS_INACTIVE),
            AuthUserStatus::Suspended => write!(f, "{}", AUTH_USER_STATUS_SUSPENDED),
            AuthUserStatus::Unknown => write!(f, "{}", AUTH_USER_STATUS_UNKNOWN),
            AuthUserStatus::Custom(c) => write!(f, "{}", c),
        }
    }
}

impl Default for AuthUserStatus {
    fn default() -> Self {
        Self::Pending
    }
}
