use dirtybase_db::{
    base::{
        helper::to_fk_column,
        manager::Manager,
        table::{
            CREATED_AT_FIELD, CREATOR_FIELD, DELETED_AT_FIELD, EDITOR_FIELD, UPDATED_AT_FIELD,
        },
    },
    entity::user::USER_TABLE,
};

mod role_user_entity;
mod role_user_repository;
mod role_user_service;

use dirtybase_db_types::TableEntityTrait;
pub use role_user_entity::RoleUserEntity;
pub use role_user_repository::RoleUserRepository;
pub use role_user_service::RoleUserService;

use super::role::ROLE_TABLE;

// Table
pub const ROLE_USER_TABLE: &str = "core_role_user";

// Field
pub const ROLE_USER_TABLE_CORE_APP_ROLE_ID_FIELD: &str = "core_app_role_id";
pub const ROLE_USER_TABLE_CORE_USER_ID_FIELD: &str = "core_user_id";
pub const ROLE_USER_TABLE_CREATOR_FIELD: &str = CREATOR_FIELD;
pub const ROLE_USER_TABLE_EDITOR_FIELD: &str = EDITOR_FIELD;
pub const ROLE_USER_TABLE_CREATED_AT_FIELD: &str = CREATED_AT_FIELD;
pub const ROLE_USER_TABLE_UPDATED_AT_FIELD: &str = UPDATED_AT_FIELD;
pub const ROLE_USER_TABLE_DELETED_AT_FIELD: &str = DELETED_AT_FIELD;

/// Creates the role users table
/// This table requires the `user` and `role` table
///
pub async fn setup_role_users_table(manager: &Manager) {
    if !manager.has_table(USER_TABLE).await {
        log::error!(
            "{} is require to create {} table",
            USER_TABLE,
            ROLE_USER_TABLE
        );
    }

    if !manager.has_table(ROLE_TABLE).await {
        log::error!(
            "{} is require to create {} table",
            ROLE_TABLE,
            ROLE_USER_TABLE
        );
    }
    manager
        .create_table_schema(RoleUserEntity::table_name(), |table| {
            // role id
            table.ulid_fk(ROLE_TABLE, true);
            // user id
            table.ulid_fk(USER_TABLE, true);
            // blame
            table.blame();
            // timestamps
            table.timestamps();
            // soft delete
            table.soft_deletable();

            // primary key
            let keys = vec![
                to_fk_column(ROLE_TABLE, None),
                to_fk_column(USER_TABLE, None),
            ];
            table.primary_index(
                keys.iter()
                    .map(AsRef::as_ref)
                    .collect::<Vec<&str>>()
                    .as_slice(),
            );
        })
        .await;
}
