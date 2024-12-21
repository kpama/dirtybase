use super::UserStatus;
use crate::db::base::helper::generate_ulid;
use dirtybase_db::types::{
    BooleanField, DateTimeField, InternalIdField, NumberField, StringField, TimestampField,
    UlidField,
};
use dirtybase_db_macro::DirtyTable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, DirtyTable)]
#[dirty(table = "core_user", id = "id")]
pub struct UserEntity {
    pub internal_id: InternalIdField,
    pub id: UlidField,
    pub username: StringField,
    pub email: StringField,
    pub reset_password: BooleanField,
    pub status: Option<UserStatus>,
    #[serde(skip_deserializing, skip_serializing)]
    pub password: StringField,
    #[serde(skip_deserializing, skip_serializing)]
    pub salt: StringField,
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
            id: Some(generate_ulid()),
            status: Some(UserStatus::Pending),
            ..Self::default()
        }
    }
}
