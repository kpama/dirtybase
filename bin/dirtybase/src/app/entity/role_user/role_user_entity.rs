use dirtybase_db::macros::DirtyTable;
use dirtybase_db_types::types::{DateTimeField, UlidField};

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(table = "core_role_user")]
pub struct RoleUserEntity {
    pub core_app_role_id: UlidField,
    pub core_user_id: UlidField,
    pub creator_id: UlidField,
    pub editor_id: UlidField,
    pub created_at: DateTimeField,
    pub updated_at: DateTimeField,
    pub deleted_at: DateTimeField,
}

impl RoleUserEntity {
    pub fn new() -> Self {
        Self::default()
    }
}
