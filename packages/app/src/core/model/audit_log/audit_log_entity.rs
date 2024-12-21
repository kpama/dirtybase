use dirtybase_contract::db::types::{DateTimeField, StringField};
use dirtybase_db_macro::DirtyTable;

#[derive(Debug, Clone, Default, DirtyTable, serde::Deserialize, serde::Serialize)]
#[dirty(table = "core_app_schema", id = "id")]
pub struct AuditLogEntity {
    pub id: StringField,
    pub event: StringField,
    pub meta: StringField,
    pub record: StringField,
    pub created_at: DateTimeField,
    pub updated_at: DateTimeField,
    pub deleted_at: DateTimeField,
}
