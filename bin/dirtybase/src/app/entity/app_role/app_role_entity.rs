use dirtybase_db::macros::DirtyTable;
use dirtybase_db_types::types::{DateTimeField, InternalIdField, StringField, UlidField};

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(table = "core_app_role")]
pub struct AppRoleEntity {
    internal_id: InternalIdField,
    id: UlidField,
    core_app_id: UlidField,
    name: StringField,
    pub creator_id: UlidField,
    pub editor_id: UlidField,
    pub created_at: DateTimeField,
    pub updated_at: DateTimeField,
    pub deleted_at: DateTimeField,
}
