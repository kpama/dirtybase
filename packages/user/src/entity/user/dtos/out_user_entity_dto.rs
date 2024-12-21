use dirtybase_contract::db::types::{DateTimeField, UlidField};
use dirtybase_db_macro::DirtyTable;
use serde::Serialize;

use crate::entity::user::{UserEntity, UserStatus};

#[derive(Debug, Clone, Serialize, Default, DirtyTable)]
pub struct OutUserEntityDto {
    pub id: UlidField,
    pub username: String,
    pub status: UserStatus,
    pub reset_password: bool,
    pub created_at: DateTimeField,
}

impl From<UserEntity> for OutUserEntityDto {
    fn from(value: UserEntity) -> Self {
        Self {
            id: value.id,
            reset_password: value.reset_password.unwrap_or_default(),
            username: value.username.unwrap_or_default(),
            created_at: value.created_at,
            status: value.status.unwrap_or_default(),
        }
    }
}
