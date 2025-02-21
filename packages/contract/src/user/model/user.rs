use crate::{
    db::types::{
        ArcUuid7, BooleanField, DateTimeField, NumberField, OptionalBooleanField,
        OptionalStringField, TimestampField,
    },
    user::status::UserStatus,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: ArcUuid7,
    pub username: String,
    pub email: OptionalStringField,
    pub reset_password: OptionalBooleanField,
    pub status: UserStatus,
    #[serde(skip_deserializing, skip_serializing)]
    pub password: OptionalStringField,
    #[serde(skip_deserializing, skip_serializing)]
    pub salt: OptionalStringField,
    pub login_attempt: NumberField,
    pub is_sys_admin: BooleanField,
    #[serde(skip_deserializing)]
    pub last_login_at: TimestampField,
    #[serde(skip_deserializing)]
    pub created_at: DateTimeField,
    #[serde(skip_deserializing)]
    pub updated_at: DateTimeField,
    #[serde(skip_deserializing)]
    pub deleted_at: DateTimeField,
}
