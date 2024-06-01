use dirtybase_contract::db::{
    macros::DirtyTable,
    types::{DateTimeField, InternalIdField, NumberField, StringField},
};

#[derive(Debug, Default, DirtyTable)]
#[dirty(table = "core_migration", id = "id")]
pub struct MigrationEntity {
    id: InternalIdField,
    pub(crate) name: StringField,
    pub(crate) batch: NumberField,
    pub(crate) created_at: DateTimeField,
}
