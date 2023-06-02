use chrono::{DateTime, Utc};

use super::UserStatus;
use crate::base::{
    field_values::FieldValue,
    helper::generate_ulid,
    types::{ColumnAndValue, FromColumnAndValue, IntoColumnAndValue},
    ColumnAndValueBuilder,
};

#[derive(Debug, Clone)]
pub struct UserEntity {
    pub internal_id: Option<u64>,
    pub id: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub reset_password: Option<bool>,
    pub status: Option<UserStatus>,
    pub password: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Default for UserEntity {
    fn default() -> Self {
        Self {
            internal_id: None,
            id: None,
            status: None,
            username: None,
            email: None,
            reset_password: None,
            password: None,
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    }
}

impl UserEntity {
    pub fn new() -> Self {
        Self {
            id: Some(generate_ulid()),
            status: Some(UserStatus::Pending),
            ..Self::default()
        }
    }
}

impl FromColumnAndValue for UserEntity {
    fn from_column_value(column_and_value: ColumnAndValue) -> Self {
        Self {
            internal_id: FieldValue::from_ref_option_into(column_and_value.get("internal_id")),
            id: FieldValue::from_ref_option_into(column_and_value.get("id")),
            username: FieldValue::from_ref_option_into(column_and_value.get("username")),
            email: FieldValue::from_ref_option_into(column_and_value.get("email")),
            reset_password: FieldValue::from_ref_option_into(
                column_and_value.get("reset_password"),
            ),
            status: FieldValue::from_ref_option_into(column_and_value.get("status")),
            password: FieldValue::from_ref_option_into(column_and_value.get("password")),
            created_at: FieldValue::from_ref_option_into(column_and_value.get("created_at")),
            updated_at: FieldValue::from_ref_option_into(column_and_value.get("created_at")),
            deleted_at: FieldValue::from_ref_option_into(column_and_value.get("created_at")),
        }
    }
}

impl IntoColumnAndValue for UserEntity {
    fn into_column_value(self) -> crate::base::types::ColumnAndValue {
        ColumnAndValueBuilder::new()
            .try_to_insert("id", self.id)
            .try_to_insert("username", self.username)
            .try_to_insert("email", self.email)
            .try_to_insert("reset_password", self.reset_password)
            .try_to_insert("status", self.status)
            .try_to_insert("password", self.password)
            .build()
    }
}
