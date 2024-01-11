use dirtybase_contract::db::{
    macros::DirtyTable,
    types::{DateTimeField, InternalIdField, StringField, UnsignedIntegerField},
};

#[derive(Debug, Default, DirtyTable)]
#[dirty(table = "core_migration", id = "id")]
pub struct MigrationEntity {
    id: InternalIdField,
    pub(crate) name: StringField,
    pub(crate) batch: UnsignedIntegerField,
    pub(crate) created_at: DateTimeField,
}
