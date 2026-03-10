use dirtybase_common::db::TableModel;
use dirtybase_common::db::types::StatusField;
use dirtybase_contract::anyhow;
use dirtybase_contract::auth_contract::{Actor, ActorRole, Permission, Role, RolePermission};
use dirtybase_contract::db_contract::base::manager::Manager;
use dirtybase_contract::db_contract::migration::Migration;

pub struct Mig1762480990CreatePermissionTables;

#[dirtybase_contract::async_trait]
impl Migration for Mig1762480990CreatePermissionTables {
    async fn up(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        // Role table
        _ = manager
            .create_table_schema(Role::table_name(), |bp| {
                bp.uuid_as_id(Some(Role::id_column()));
                bp.string(Role::col_name_for_name());
                bp.string(Role::col_name_for_label());
                bp.sized_string(Role::col_name_for_description(), 512);
                bp.timestamps();
                bp.soft_deletable();
                bp.unique_index(&[Role::col_name_for_name()]);
            })
            .await?;
        // Permission table
        _ = manager
            .create_table_schema(Permission::table_name(), |bp| {
                bp.uuid_as_id(Some(Permission::id_column()));
                bp.string(Permission::col_name_for_label());
                bp.string(Permission::col_name_for_name())
                    .set_is_unique(true);
                bp.sized_string(Permission::col_name_for_description(), 512);
                bp.text(Permission::col_name_for_condition())
                    .set_is_nullable(true);
                bp.timestamps();
                bp.soft_deletable();
                bp.unique_index(&[Permission::col_name_for_name()]);
            })
            .await?;

        // Actor-Role table
        _ = manager
            .create_table_schema(ActorRole::table_name(), |bp| {
                bp.uuid_as_id(Some(ActorRole::id_column()));
                bp.uuid_table_fk::<Actor>(true);
                bp.uuid_table_fk::<Role>(true);
                bp.string(ActorRole::col_name_for_status())
                    .set_default(StatusField::Pending);
                bp.timestamps();
                bp.soft_deletable();
                bp.unique_index(&[Actor::foreign_id_column(), Role::foreign_id_column()]);
            })
            .await?;
        // Role/Actor-Permission table
        _ = manager
            .create_table_schema(RolePermission::table_name(), |bp| {
                bp.uuid_as_id(Some(RolePermission::id_column()));
                bp.uuid_table_fk::<Permission>(true);
                bp.uuid_table_fk::<Role>(true).nullable();
                bp.uuid_table_fk::<Actor>(true).nullable();
                bp.string(RolePermission::col_name_for_status())
                    .set_default(StatusField::Pending);
                bp.timestamps();
                bp.soft_deletable();
                bp.unique_index(&[
                    RolePermission::col_name_for_auth_permission_id(),
                    RolePermission::col_name_for_auth_role_id(),
                    RolePermission::col_name_for_auth_actor_id(),
                ]);
            })
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        _ = manager.drop_table(RolePermission::table_name()).await?;
        _ = manager.drop_table(ActorRole::table_name()).await?;
        _ = manager.drop_table(Permission::table_name()).await?;
        _ = manager.drop_table(Role::table_name()).await?;

        #[cfg(feature = "permission")]
        {
            _ = manager.drop_table(Actor::table_name()).await?;
        }

        Ok(())
    }
}
