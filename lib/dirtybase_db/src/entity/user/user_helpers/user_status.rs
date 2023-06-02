use crate::base::field_values::FieldValue;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(untagged)]
pub enum UserStatus {
    #[serde(rename(serialize = "active"))]
    Active,
    #[serde(rename(serialize = "inactive"))]
    Inactive,
    #[serde(rename(serialize = "pending"))]
    Pending,
    #[serde(rename(serialize = "suspended"))]
    Suspended,
    #[serde(rename(serialize = "unknown"))]
    Unknown,
}

pub const USER_STATUS_ACTIVE: &str = "active";
pub const USER_STATUS_INACTIVE: &str = "inactive";
pub const USER_STATUS_PENDING: &str = "pending";
pub const USER_STATUS_SUSPENDED: &str = "suspended";
pub const USER_STATUS_UNKNOWN: &str = "unknown";

impl From<UserStatus> for FieldValue {
    fn from(value: UserStatus) -> Self {
        value.to_string().into()
    }
}

impl From<&UserStatus> for FieldValue {
    fn from(value: &UserStatus) -> Self {
        value.to_string().into()
    }
}

impl From<FieldValue> for UserStatus {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(s) => match s.to_ascii_lowercase().as_str() {
                USER_STATUS_ACTIVE => Self::Active,
                USER_STATUS_INACTIVE => Self::Inactive,
                USER_STATUS_SUSPENDED => Self::Suspended,
                USER_STATUS_UNKNOWN => Self::Pending,
                _ => Self::Unknown,
            },
            _ => Self::Unknown,
        }
    }
}

impl From<&FieldValue> for UserStatus {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::String(v) => FieldValue::from(v).into(),
            _ => UserStatus::Unknown,
        }
    }
}

impl From<FieldValue> for Option<UserStatus> {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(s) => match s.to_ascii_lowercase().as_str() {
                USER_STATUS_ACTIVE => Some(UserStatus::Active),
                USER_STATUS_INACTIVE => Some(UserStatus::Inactive),
                USER_STATUS_SUSPENDED => Some(UserStatus::Suspended),
                USER_STATUS_PENDING => Some(UserStatus::Pending),
                USER_STATUS_UNKNOWN => Some(UserStatus::Unknown),
                _ => None,
            },
            _ => None,
        }
    }
}

impl Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserStatus::Pending => write!(f, "{}", USER_STATUS_PENDING),
            UserStatus::Active => write!(f, "{}", USER_STATUS_ACTIVE),
            UserStatus::Inactive => write!(f, "{}", USER_STATUS_INACTIVE),
            UserStatus::Suspended => write!(f, "{}", USER_STATUS_SUSPENDED),
            UserStatus::Unknown => write!(f, "{}", USER_STATUS_UNKNOWN),
        }
    }
}
