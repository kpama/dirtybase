mod role_permission_entity;

use dirtybase_db::TableModel;
use dirtybase_db::base::manager::Manager;
pub use role_permission_entity::RolePermissionEntity;

use super::{permission::permission_entity::PermissionEntity, role::RoleEntity};

pub async fn setup_role_permission_table(manager: &Manager) {
    // TODO: check for required tables

    manager
        .create_table_schema(RolePermissionEntity::table_name(), |table| {
            // role id
            table.ulid_fk(RoleEntity::table_name(), true);

            // permission id
            table.ulid_fk(PermissionEntity::table_name(), true);

            // blame
            table.blame();

            // timestamps
            table.timestamps();
            // soft delete
            table.soft_deletable();

            // primary key
            table.primary_index(&[
                RoleEntity::foreign_id_column().unwrap(),
                PermissionEntity::foreign_id_column().unwrap(),
            ]);
        })
        .await;
}
