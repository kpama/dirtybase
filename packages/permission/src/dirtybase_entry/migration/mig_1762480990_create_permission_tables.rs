use dirtybase_common::db::TableModel;
use dirtybase_contract::anyhow;
use dirtybase_contract::auth_contract::AuthUser;
use dirtybase_contract::db_contract::base::manager::Manager;
use dirtybase_contract::db_contract::migration::Migration;
use dirtybase_contract::prelude::model::{
    Actor, ActorRole, Permission, Role, RolePermission, Tenant,
};

pub struct Mig1762480990CreatePermissionTables;

#[dirtybase_contract::async_trait]
impl Migration for Mig1762480990CreatePermissionTables {
    async fn up(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        // Tenant table
        _ = manager
            .create_table_schema(Tenant::table_name(), |bp| {
                bp.uuid_as_id(Some(Tenant::id_column()));
                bp.string(Tenant::col_name_for_name());
                bp.sized_string(Tenant::col_name_for_domain(), 512);
                bp.boolean(Tenant::col_name_for_is_global());
                bp.timestamps();
                bp.soft_deletable();
            })
            .await?;
        // Actor table
        _ = manager
            .create_table_schema(Actor::table_name(), |bp| {
                bp.uuid_as_id(Some(Actor::id_column()));
                bp.uuid_table_fk::<AuthUser>(true).nullable();
                bp.timestamps();
                bp.soft_deletable();
            })
            .await?;
        // Role table
        _ = manager
            .create_table_schema(Role::table_name(), |bp| {
                bp.uuid_as_id(Some(Role::id_column()));
                bp.string(Role::col_name_for_name());
                bp.string(Role::col_name_for_label());
                bp.sized_string(Role::col_name_for_description(), 512);
                bp.timestamps();
                bp.soft_deletable();
            })
            .await?;
        // Permission table
        _ = manager
            .create_table_schema(Permission::table_name(), |bp| {
                bp.uuid_as_id(Some(Permission::id_column()));
                bp.string(Permission::col_name_for_label());
                bp.string(Permission::col_name_for_name());
                bp.sized_string(Permission::col_name_for_description(), 512);
                bp.timestamps();
                bp.soft_deletable();
            })
            .await?;

        // Actor-Role table
        _ = manager
            .create_table_schema(ActorRole::table_name(), |bp| {
                bp.uuid_as_id(Some(ActorRole::id_column()));
                bp.uuid_table_fk::<Actor>(true);
                bp.uuid_table_fk::<Role>(true);
                bp.uuid_table_fk::<Tenant>(true).nullable();
                bp.timestamps();
                bp.soft_deletable();
            })
            .await?;
        // Role/Actor-Permission table
        _ = manager
            .create_table_schema(RolePermission::table_name(), |bp| {
                bp.uuid_as_id(Some(RolePermission::id_column()));
                bp.uuid_table_fk::<Permission>(true);
                bp.uuid_table_fk::<Role>(true).nullable();
                bp.uuid_table_fk::<Actor>(true).nullable();
                bp.uuid_table_fk::<Tenant>(true).nullable();
                bp.timestamps();
                bp.soft_deletable();
                bp.unique_index(&[
                    RolePermission::col_name_for_perm_permission_id(),
                    RolePermission::col_name_for_perm_role_id(),
                    RolePermission::col_name_for_perm_actor_id(),
                    RolePermission::col_name_for_perm_tenant_id(),
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
        _ = manager.drop_table(Actor::table_name()).await?;
        _ = manager.drop_table(Tenant::table_name()).await?;
        Ok(())
    }
}
