use dirtybase_db::base::{
    manager::Manager,
    table::{
        CREATED_AT_FIELD, CREATOR_FIELD, DELETED_AT_FIELD, EDITOR_FIELD, ID_FIELD,
        INTERNAL_ID_FIELD, UPDATED_AT_FIELD,
    },
};

mod app_entity;
mod app_repository;
mod app_service;

pub use app_entity::AppEntity;
pub use app_repository::AppRepository;
pub use app_service::AppService;
use dirtybase_db::TableEntityTrait;

use super::company::COMPANY_TABLE;

// Table
pub const APP_TABLE: &str = "core_app";

// Fields
pub const APP_TABLE_NAME_FIELD: &str = "name";
pub const APP_TABLE_DESCRIPTION_FIELD: &str = "description";
pub const APP_TABLE_IS_SYSTEM_APP_FIELD: &str = "is_system_app";
pub const APP_TABLE_COMPANY_ID_FIELD: &str = "core_company_id";
pub const APP_TABLE_INTERNAL_ID_FIELD: &str = INTERNAL_ID_FIELD;
pub const APP_TABLE_ID_FIELD: &str = ID_FIELD;
pub const APP_TABLE_CREATOR_FIELD: &str = CREATOR_FIELD;
pub const APP_TABLE_EDITOR_FIELD: &str = EDITOR_FIELD;
pub const APP_TABLE_CREATED_AT_FIELD: &str = CREATED_AT_FIELD;
pub const APP_TABLE_UPDATED_AT_FIELD: &str = UPDATED_AT_FIELD;
pub const APP_TABLE_DELETED_AT_FIELD: &str = DELETED_AT_FIELD;

pub async fn setup_applications_table(manager: &Manager) {
    manager
        .create_table_schema(AppEntity::table_name(), |table| {
            // internal_id
            // id
            table.id_set();
            // company_id
            table.ulid_fk(COMPANY_TABLE, true);
            // is_system_app
            // This field identifies the main system application
            table
                .boolean(APP_TABLE_IS_SYSTEM_APP_FIELD)
                .set_default_from(false);
            // name
            table.string(APP_TABLE_NAME_FIELD);
            //description
            table
                .sized_string(APP_TABLE_DESCRIPTION_FIELD, 512)
                .set_is_nullable(true);
            // blame
            table.blame();
            // timestamp
            table.timestamps();
            // soft delete
            table.soft_deletable();
        })
        .await
}
