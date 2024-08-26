use crate::core::model::company::CompanyEntity;
use dirtybase_contract::db::base::helper::generate_ulid;
use dirtybase_contract::db::types::{
    BooleanField, DateTimeField, InternalIdField, StringField, UlidField,
};
use dirtybase_db_macro::DirtyTable;

#[derive(Debug, Clone, Default, DirtyTable, serde::Deserialize, serde::Serialize)]
#[dirty(table = "core_app", id = "id")]
pub struct AppEntity {
    pub internal_id: InternalIdField,
    pub id: UlidField,
    pub name: StringField,
    pub description: StringField,
    pub is_system_app: BooleanField,
    pub core_company_id: UlidField,
    #[dirty(skip_select, skip_insert)]
    pub company: CompanyEntity,
    pub creator_id: UlidField,
    pub editor_id: UlidField,
    pub created_at: DateTimeField,
    pub updated_at: DateTimeField,
    pub deleted_at: DateTimeField,
}

impl AppEntity {
    pub fn new() -> Self {
        Self {
            id: Some(generate_ulid()),
            ..Self::default()
        }
    }
}
