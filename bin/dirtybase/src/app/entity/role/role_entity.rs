use dirtybase_db::{
    base::helper::generate_ulid, dirtybase_db_types::types::InternalIdField, macros::DirtyTable,
};
use dirtybase_db_types::types::{DateTimeField, StringField, UlidField};

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(table = "core_app_role")]
pub struct RoleEntity {
    pub internal_id: InternalIdField,
    pub id: UlidField,
    pub name: StringField,
    pub core_app_id: UlidField,
    pub creator_id: UlidField,
    pub editor_id: UlidField,
    pub created_at: DateTimeField,
    pub updated_at: DateTimeField,
    pub deleted_at: DateTimeField,
}

impl RoleEntity {
    pub fn new() -> Self {
        Self {
            id: Some(generate_ulid()),
            ..Self::default()
        }
    }
}
