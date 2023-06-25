use super::entity::{
    app::setup_applications_table, audit_log::setup_audit_log_table, company::setup_company_table,
    dirtybase_user::setup_dirtybase_user_table, role::setup_roles_table,
    role_user::setup_role_users_table, sys_admin::setup_sysadmins_table,
};
use dirtybase_db::{base::manager::Manager, entity::user::setup_users_table};

pub const APPLICATION_TABLE: &str = "core_app";
pub const APPLICATION_SCHEMA_TABLE: &str = "core_app_schema";
pub const MIGRATION_TABLE: &str = "core_migration";
pub const FILE_METADATA_TABLE: &str = "core_file_meta";

// The table that will hold company's tenets

// The table that will contain the "collections" definitions
async fn setup_schema_table(manager: &Manager) {
    manager
        .create_table_schema(APPLICATION_SCHEMA_TABLE, |table| {
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
async fn setup_migration_table(manager: &Manager) {
    manager
        .create_table_schema(MIGRATION_TABLE, |table| {
            // id
            table.id(None);
            // migration name
            table.string("name");
            // created at
            table.created_at();
            // deleted at
            table.updated_at();
        })
        .await;
}

// The table that will hold file metadata
async fn setup_file_metadata_table(manager: &Manager) {
    manager
        .create_table_schema(FILE_METADATA_TABLE, |table| {
            // internal_id
            // id
            table.id_set();
            // external_id
            table.ulid("external_id").set_is_nullable(false);
            // meta
            table.json("meta");
            // timestamp
            table.timestamps();
        })
        .await;
}

pub(crate) async fn create_data_tables(manager: Manager) {
    setup_users_table(&manager).await;
    setup_migration_table(&manager).await;
    setup_file_metadata_table(&manager).await;
    setup_audit_log_table(&manager).await;
    setup_company_table(&manager).await;
    setup_applications_table(&manager).await;
    setup_schema_table(&manager).await;
    setup_roles_table(&manager).await;
    setup_role_users_table(&manager).await;
    setup_sysadmins_table(&manager).await;
    setup_dirtybase_user_table(&manager).await;
}
