use dirtybase_db::{dirtybase_db_types::types::InternalIdField, macros::DirtyTable};
use dirtybase_db_types::types::{DateTimeField, StringField, UnsignedIntegerField};

#[derive(Debug, Default, DirtyTable)]
#[dirty(table = "core_migration", id = "id")]
pub struct MigrationEntity {
    id: InternalIdField,
    pub(crate) name: StringField,
    pub(crate) batch: UnsignedIntegerField,
    pub(crate) created_at: DateTimeField,
}
