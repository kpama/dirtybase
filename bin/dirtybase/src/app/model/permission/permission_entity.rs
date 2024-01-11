use dirtybase_contract::db::entity::user::UserEntity;
use dirtybase_contract::db::macros::DirtyTable;

use dirtybase_db::types::{DateTimeField, InternalIdField, StringField, UlidField};

use crate::app::model::company::CompanyEntity;

#[derive(Debug, Clone, Default, DirtyTable, serde::Serialize, serde::Deserialize)]
#[dirty(table = "core_permission", id = "id")]
pub struct PermissionEntity {
    pub internal_id: InternalIdField,
    pub id: UlidField,
    pub name: StringField,
    pub label: StringField,
    pub description: StringField,

    #[dirty(col = "core_company_id", skip_select)]
    pub company: Option<CompanyEntity>,
    #[dirty(col = "creator_id", skip_select)]
    pub creator: Option<UserEntity>,
    #[dirty(col = "creator_id", skip_select)]
    pub editor: Option<UserEntity>,

    #[dirty(skip_select)]
    pub core_company_id: UlidField,
    #[dirty(skip_select)]
    pub creator_id: UlidField,
    #[dirty(skip_select)]
    pub editor_id: UlidField,

    pub created_at: DateTimeField,
    pub updated_at: DateTimeField,
    pub deleted_at: DateTimeField,
}
