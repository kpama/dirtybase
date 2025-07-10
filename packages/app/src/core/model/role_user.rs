use dirtybase_contract::db::base::{helper::to_fk_column, manager::Manager};

use dirtybase_user::entity::user::UserEntity;

mod role_user_entity;
mod role_user_repository;
mod role_user_service;

use dirtybase_contract::db::TableModel;
pub use role_user_entity::RoleUserEntity;
pub use role_user_repository::RoleUserRepository;
pub use role_user_service::RoleUserService;

use super::role::RoleEntity;

/// Creates the role users table
/// This table requires the `user` and `role` table
///
pub async fn setup_role_users_table(manager: &Manager) {
    if !check_for_required_tables(manager).await {
        return;
    }

    manager
        .create_table_schema(RoleUserEntity::table_name(), |table| {
            // role id
            table.ulid_fk(RoleEntity::table_name(), true);
            // user id
            table.ulid_fk(UserEntity::table_name(), true);
            // blame
            table.blame();
            // timestamps
            table.timestamps();
            // soft delete
            table.soft_deletable();

            // primary key
            let keys = [
                to_fk_column(RoleEntity::table_name(), None),
                to_fk_column(UserEntity::table_name(), None),
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

async fn check_for_required_tables(manager: &Manager) -> bool {
    let table_names = [UserEntity::table_name(), RoleEntity::table_name()];

    for name in table_names {
        if !manager.has_table(name).await {
            log::error!(
                "{} is require to create {} table",
                name,
                RoleUserEntity::table_name()
            );
            return false;
        }
    }

    true
}
