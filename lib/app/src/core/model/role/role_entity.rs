use dirtybase_contract::db::{
    base::helper::generate_ulid,
    macros::DirtyTable,
    types::{DateTimeField, InternalIdField, StringField, UlidField},
};

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(table = "core_app_role", id = "id")]
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
