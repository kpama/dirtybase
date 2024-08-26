use dirtybase_contract::db::types::{DateTimeField, InternalIdField, NumberField, StringField};
use dirtybase_db_macro::DirtyTable;

#[derive(Debug, Default, DirtyTable, Clone)]
#[dirty(table = "core_migration", id = "id")]
pub struct MigrationEntity {
    id: InternalIdField,
    pub(crate) name: StringField,
    pub(crate) batch: NumberField,
    pub(crate) created_at: DateTimeField,
}
