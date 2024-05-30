use dirtybase_contract::db::types::{DateTimeField, InternalIdField, StringField, UlidField};
use dirtybase_contract::db::{entity::user::UserEntity, macros::DirtyTable};

use crate::core::model::app_entity::AppEntity;

#[derive(Debug, Clone, Default, DirtyTable, serde::Deserialize, serde::Serialize)]
#[dirty(table = "core_app_schema", id = "id")]
pub struct AppSchemaEntity {
    pub internal_id: InternalIdField,
    pub id: UlidField,
    #[dirty(col = "core_app_id", skip_select)]
    pub app: Option<AppEntity>,
    #[dirty(skip_select)]
    pub core_app_id: UlidField,
    pub table_name: StringField,
    pub table_definition: StringField,
    #[dirty(col = "creator_id", skip_select)]
    pub creator: Option<UserEntity>,
    #[dirty(skip_select)]
    pub creator_id: UlidField,
    pub editor_id: UlidField,
    pub created_at: DateTimeField,
    pub updated_at: DateTimeField,
    pub deleted_at: DateTimeField,
}
