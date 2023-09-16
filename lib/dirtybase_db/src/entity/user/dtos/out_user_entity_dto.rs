use dirtybase_db_macro::DirtyTable;
use dirtybase_db_types::types::DateTimeField;
use serde::Serialize;

use crate::entity::user::{UserEntity, UserStatus};

#[derive(Debug, Clone, Serialize, Default, DirtyTable)]
pub struct OutUserEntityDto {
    pub id: String,
    pub username: String,
    pub status: UserStatus,
    pub reset_password: bool,
    pub created_at: DateTimeField,
}

impl From<UserEntity> for OutUserEntityDto {
    fn from(value: UserEntity) -> Self {
        Self {
            id: value.id.unwrap_or_default(),
            reset_password: value.reset_password.unwrap_or_default(),
            username: value.username.unwrap_or_default(),
            created_at: if let Some(dt) = value.created_at {
                Some(dt)
            } else {
                None
            },
            status: value.status.unwrap_or_default(),
        }
    }
}
