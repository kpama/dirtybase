use super::UserStatus;
use crate::base::helper::generate_ulid;
use dirtybase_db_macro::DirtyTable;
use dirtybase_db_types::types::{
    BooleanField, DateTimeField, InternalIdField, StringField, UlidField,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, DirtyTable)]
#[dirty(table = "core_user")]
pub struct UserEntity {
    pub internal_id: InternalIdField,
    pub id: UlidField,
    pub username: StringField,
    pub email: StringField,
    pub reset_password: BooleanField,
    pub status: Option<UserStatus>,
    pub password: StringField,
    #[serde(skip_deserializing)]
    pub created_at: DateTimeField,
    #[serde(skip_deserializing)]
    pub updated_at: DateTimeField,
    #[serde(skip_deserializing)]
    pub deleted_at: DateTimeField,
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
