use dirtybase_contract::db::types::{DateTimeField, InternalIdField, StringField};
use dirtybase_db::types::IntegerField;
use dirtybase_db_macro::DirtyTable;

#[derive(Debug, Default, DirtyTable, Clone)]
#[dirty(table = "core_migration", id = "id")]
pub struct MigrationEntity {
    id: InternalIdField,
    pub(crate) name: StringField,
    pub(crate) batch: IntegerField,
    pub(crate) created_at: DateTimeField,
}
