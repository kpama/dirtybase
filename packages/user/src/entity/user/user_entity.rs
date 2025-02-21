use super::UserStatus;
use dirtybase_contract::db::types::{
    ArcUuid7, BooleanField, DateTimeField, InternalIdField, NumberField, OptionalBooleanField,
    OptionalStringField, TimestampField,
};
use dirtybase_db_macro::DirtyTable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, DirtyTable)]
#[dirty(table = "users", id = "id")]
pub struct UserEntity {
    pub internal_id: InternalIdField,
    pub id: ArcUuid7,
    pub username: OptionalStringField,
    pub email: OptionalStringField,
    pub reset_password: OptionalBooleanField,
    pub status: Option<UserStatus>,
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

impl UserEntity {
    pub fn new() -> Self {
        Self {
            status: Some(UserStatus::Pending),
            ..Self::default()
        }
    }
}
