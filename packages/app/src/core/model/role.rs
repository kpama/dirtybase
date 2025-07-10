use dirtybase_contract::db::base::manager::Manager;
use dirtybase_db::TableModel;
use dirtybase_user::entity::user::UserEntity;

use super::app_entity::AppEntity;

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

/// Creates the roles table
/// The role table expects the `user` and  `app` tables to already exist
///
pub async fn setup_roles_table(manager: &Manager) {
    if !manager.has_table(UserEntity::table_name()).await {
        log::error!(
            "{} is require to create {} table",
            UserEntity::table_name(),
            RoleEntity::table_name()
        );
        eprintln!(
            "{} is require to create {} table",
            UserEntity::table_name(),
            RoleEntity::table_name()
        );
    }

    if !manager.has_table(AppEntity::table_name()).await {
        log::error!(
            "{} is require to create {} table",
            AppEntity::table_name(),
            RoleEntity::table_name()
        );
        eprintln!(
            "{} is require to create {} table",
            AppEntity::table_name(),
            RoleEntity::table_name()
        );
    }

    manager
        .create_table_schema(RoleEntity::table_name(), |table| {
            // internal_id
            // id
            table.id_set();
            // application
            table.ulid_fk(AppEntity::table_name(), true);
            // name
            table.string(RoleEntity::col_name_for_name());
            // blame
            table.blame();
            // timestamps
            table.timestamps();
            // soft delete
            table.soft_deletable();

            // name+app_id is unique
            table.unique_index(&[
                RoleEntity::col_name_for_name(),
                RoleEntity::col_name_for_core_app_id(),
            ]);
        })
        .await;
}
