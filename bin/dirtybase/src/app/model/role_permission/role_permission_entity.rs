use dirtybase_contract::db::types::{DateTimeField, UlidField};
use dirtybase_contract::db::{entity::user::UserEntity, macros::DirtyTable};

#[derive(Debug, Clone, Default, DirtyTable, serde::Serialize, serde::Deserialize)]
#[dirty(table = "core_role_permission")]
pub struct RolePermissionEntity {
    core_app_role_id: UlidField,
    core_permission: UlidField,

    #[dirty(col = "creator_id", skip_select)]
    pub creator: Option<UserEntity>,
    #[dirty(col = "creator_id", skip_select)]
    pub editor: Option<UserEntity>,

    #[dirty(skip_select)]
    pub creator_id: UlidField,
    #[dirty(skip_select)]
    pub editor_id: UlidField,

    pub created_at: DateTimeField,
    pub updated_at: DateTimeField,
    pub deleted_at: DateTimeField,
}
