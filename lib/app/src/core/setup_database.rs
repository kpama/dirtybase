use super::model::{
    app_entity::{setup_applications_table, AppEntity},
    app_schema::AppSchemaEntity,
    audit_log::{setup_audit_log_table, AuditLogEntity},
    company::{setup_company_table, CompanyEntity},
    dirtybase_user::{setup_dirtybase_user_table, DirtybaseUserEntity},
    migration::MigrationEntity,
    permission::{permission_entity::PermissionEntity, setup_permission_table},
    role::{setup_roles_table, RoleEntity},
    role_permission::{setup_role_permission_table, RolePermissionEntity},
    role_user::{setup_role_users_table, RoleUserEntity},
    sys_admin::{setup_sysadmins_table, SysAdminEntity},
};

use dirtybase_contract::db::{
    base::manager::Manager,
    TableEntityTrait, //  entity::user::setup_users_table, entity::user::UserEntity,
};
use dirtybase_user::entity::user::{setup_users_table, UserEntity};

pub const APPLICATION_TABLE: &str = "core_app";

// The table that will contain the "collections" definitions
pub(crate) async fn setup_schema_table(manager: &Manager) {
    manager
        .create_table_schema(AppEntity::table_name(), |table| {
            // internal_id
            // id
            table.id_set();
            // application ID
            table.ulid_fk(APPLICATION_TABLE, true);
            // table/collection name
            table.string("table_name");
            // table/collection definition
            table.json("table_definition");
            // blame
            table.blame();
            // timestamp
            table.timestamps();
        })
        .await
}

// The global roles table

// A user role
// System administrator table

// The table that will hold migration information
pub(crate) async fn setup_migration_table(manager: &Manager) {
    manager
        .create_table_schema(MigrationEntity::table_name(), |table| {
            // id
            table.id(Some(MigrationEntity::col_name_for_id()));

            // migration name
            table.text(MigrationEntity::col_name_for_name());

            // batch
            table.integer(MigrationEntity::col_name_for_batch());

            // created at
            table.created_at();
        })
        .await;
}

pub(crate) async fn create_default_tables(manager: &Manager) {
    setup_migration_table(manager).await;
    setup_users_table(manager).await;
    setup_audit_log_table(manager).await;
    setup_company_table(manager).await;
    setup_applications_table(manager).await;
    setup_schema_table(manager).await;
    setup_roles_table(manager).await;
    setup_role_users_table(manager).await;
    setup_sysadmins_table(manager).await;
    setup_dirtybase_user_table(manager).await;
    setup_permission_table(manager).await;
    setup_role_permission_table(manager).await;
}

pub(crate) async fn drop_default_tables(manager: &Manager) {
    let tables = [
        RolePermissionEntity::table_name(),
        PermissionEntity::table_name(),
        DirtybaseUserEntity::table_name(),
        SysAdminEntity::table_name(),
        RoleUserEntity::table_name(),
        RoleEntity::table_name(),
        AppSchemaEntity::table_name(),
        AppEntity::table_name(),
        CompanyEntity::table_name(),
        AuditLogEntity::table_name(),
        UserEntity::table_name(),
        MigrationEntity::table_name(),
    ];

    for name in tables {
        manager.drop_table(name).await;
    }
}
