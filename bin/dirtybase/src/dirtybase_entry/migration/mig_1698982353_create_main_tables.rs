use dirtybase_contract::db::base::manager::Manager;
use dirtybase_contract::db::migration::Migration;
use dirtybase_db::entity::user::setup_users_table;

use crate::app::{
    model::{
        app::setup_applications_table, audit_log::setup_audit_log_table,
        company::setup_company_table, dirtybase_user::setup_dirtybase_user_table,
        permission::setup_permission_table, role::setup_roles_table,
        role_permission::setup_role_permission_table, role_user::setup_role_users_table,
        sys_admin::setup_sysadmins_table,
    },
    setup_database::{
        create_data_tables, setup_file_metadata_table, setup_migration_table, setup_schema_table,
    },
    setup_defaults::setup_default_entities,
};

pub struct Mig1698982353createmaintables;

#[dirtybase_contract::async_trait]
impl Migration for Mig1698982353createmaintables {
    async fn up(&self, manager: &Manager) {
        create_data_tables(manager).await;
        setup_default_entities().await;
    }

    async fn down(&self, manager: &Manager) {
        println!("This is a test going down");
    }
}
