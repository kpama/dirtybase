use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{
    UserStatus, USER_TABLE_CREATED_AT_FIELD, USER_TABLE_DELETED_AT_FIELD, USER_TABLE_EMAIL_FIELD,
    USER_TABLE_ID_FIELD, USER_TABLE_INTERNAL_ID_FIELD, USER_TABLE_PASSWORD_FIELD,
    USER_TABLE_RESET_PASSWORD_FIELD, USER_TABLE_STATUS_FIELD, USER_TABLE_UPDATED_AT_FIELD,
    USER_TABLE_USERNAME_FIELD,
};
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
            internal_id: FieldValue::from_ref_option_into(
                column_and_value.get(USER_TABLE_INTERNAL_ID_FIELD),
            ),
            id: FieldValue::from_ref_option_into(column_and_value.get(USER_TABLE_ID_FIELD)),
            username: FieldValue::from_ref_option_into(
                column_and_value.get(USER_TABLE_USERNAME_FIELD),
            ),
            email: FieldValue::from_ref_option_into(column_and_value.get(USER_TABLE_EMAIL_FIELD)),
            reset_password: FieldValue::from_ref_option_into(
                column_and_value.get(USER_TABLE_RESET_PASSWORD_FIELD),
            ),
            status: FieldValue::from_ref_option_into(column_and_value.get(USER_TABLE_STATUS_FIELD)),
            password: FieldValue::from_ref_option_into(
                column_and_value.get(USER_TABLE_PASSWORD_FIELD),
            ),
            created_at: FieldValue::from_ref_option_into(
                column_and_value.get(USER_TABLE_CREATED_AT_FIELD),
            ),
            updated_at: FieldValue::from_ref_option_into(
                column_and_value.get(USER_TABLE_UPDATED_AT_FIELD),
            ),
            deleted_at: FieldValue::from_ref_option_into(
                column_and_value.get(USER_TABLE_DELETED_AT_FIELD),
            ),
        }
    }
}

impl IntoColumnAndValue for UserEntity {
    fn into_column_value(self) -> crate::base::types::ColumnAndValue {
        ColumnAndValueBuilder::new()
            .try_to_insert(USER_TABLE_ID_FIELD, self.id)
            .try_to_insert(USER_TABLE_USERNAME_FIELD, self.username)
            .try_to_insert(USER_TABLE_EMAIL_FIELD, self.email)
            .try_to_insert(USER_TABLE_PASSWORD_FIELD, self.password)
            .try_to_insert(USER_TABLE_STATUS_FIELD, self.status)
            .try_to_insert(USER_TABLE_RESET_PASSWORD_FIELD, self.reset_password)
            .try_to_insert(USER_TABLE_DELETED_AT_FIELD, self.deleted_at)
            .build()
    }
}
