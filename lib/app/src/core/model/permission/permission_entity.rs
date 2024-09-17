use std::fmt::{Debug, Display};

use dirtybase_db::types::OptionalStringField;
use dirtybase_db_macro::DirtyTable;
use dirtybase_user::entity::user::UserEntity;

use dirtybase_contract::db::types::{DateTimeField, InternalIdField, StringField, UlidField};

use crate::core::model::company::CompanyEntity;

pub struct PermissionName(String);

impl Display for PermissionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let subject = self.0.split(' ').map(str::to_lowercase).collect::<String>();
        write!(f, "{}", subject)
    }
}

impl Debug for PermissionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let subject = self.0.split(' ').map(str::to_lowercase).collect::<String>();
        write!(f, "{}", subject)
    }
}

#[derive(Debug, Clone, Default, DirtyTable, serde::Serialize, serde::Deserialize)]
#[dirty(table = "core_permission", id = "id")]
pub struct PermissionEntity {
    pub internal_id: InternalIdField,
    pub id: UlidField,
    pub name: OptionalStringField,
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
