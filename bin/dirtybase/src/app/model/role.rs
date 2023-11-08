use dirtybase_db::db::{
    base::{
        manager::Manager,
        table::{
            CREATED_AT_FIELD, CREATOR_FIELD, DELETED_AT_FIELD, EDITOR_FIELD, ID_FIELD,
            INTERNAL_ID_FIELD, UPDATED_AT_FIELD,
        },
    },
    entity::user::USER_TABLE,
};
use dirtybase_db_types::TableEntityTrait;

use super::app::APP_TABLE;

mod role_entity;
mod role_repository;
mod role_service;

pub mod dtos;

pub use role_entity::RoleEntity;
pub use role_repository::RoleRepository;
pub use role_service::RoleService;

// default roles
pub const ROLE_ADMIN: &str = "admin";
pub const ROLE_USER: &str = "user";

// Table
pub const ROLE_TABLE: &str = "core_app_role";

// Fields
pub const ROLE_TABLE_NAME_FIELD: &str = "name";
pub const ROLE_TABLE_APP_ID_FIELD: &str = "core_app_id";
pub const ROLE_TABLE_INTERNAL_ID_FIELD: &str = INTERNAL_ID_FIELD;
pub const ROLE_TABLE_ID_FIELD: &str = ID_FIELD;
pub const ROLE_TABLE_CREATOR_FIELD: &str = CREATOR_FIELD;
pub const ROLE_TABLE_EDITOR_FIELD: &str = EDITOR_FIELD;
pub const ROLE_TABLE_CREATED_AT_FIELD: &str = CREATED_AT_FIELD;
pub const ROLE_TABLE_UPDATED_AT_FIELD: &str = UPDATED_AT_FIELD;
pub const ROLE_TABLE_DELETED_AT_FIELD: &str = DELETED_AT_FIELD;

/// Creates the roles table
/// The role table expects the `user` and  `app` tables to already exist
///
pub async fn setup_roles_table(manager: &Manager) {
    if !manager.has_table(USER_TABLE).await {
        log::error!("{} is require to create {} table", USER_TABLE, ROLE_TABLE);
        eprintln!("{} is require to create {} table", USER_TABLE, ROLE_TABLE);
    }

    if !manager.has_table(APP_TABLE).await {
        log::error!("{} is require to create {} table", APP_TABLE, ROLE_TABLE);
        eprintln!("{} is require to create {} table", APP_TABLE, ROLE_TABLE);
    }

    manager
        .create_table_schema(RoleEntity::table_name(), |table| {
            // internal_id
            // id
            table.id_set();
            // application
            table.ulid_fk(APP_TABLE, true);
            // name
            table.string(ROLE_TABLE_NAME_FIELD);
            // blame
            table.blame();
            // timestamps
            table.timestamps();
            // soft delete
            table.soft_deletable();
        })
        .await;
}
