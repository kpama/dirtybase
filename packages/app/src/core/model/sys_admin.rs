//! Sys admin managers users that have system wild access
//!
//! The default user as setup time is placed in this list.

use dirtybase_contract::db::base::manager::Manager;

mod sys_admin_entity;
mod sys_admin_repository;
mod sys_admin_service;

use dirtybase_db::TableModel;
use dirtybase_user::entity::user::UserEntity;
pub use sys_admin_entity::SysAdminEntity;
pub use sys_admin_repository::SysAdminRepository;
pub use sys_admin_service::SysAdminService;

// Table
pub const SYS_ADMIN_TABLE: &str = "core_sys_admin";

// Field
pub const SYS_ADMIN_TABLE_USER_ID_FIELD: &str = "core_user_id";

pub async fn setup_sysadmins_table(manager: &Manager) {
    if !manager.has_table(UserEntity::table_name()).await {
        log::error!(
            "{} is require to create {} table",
            UserEntity::table_name(),
            SYS_ADMIN_TABLE
        );
    }

    manager
        .create_table_schema(SysAdminEntity::table_name(), |table| {
            // the user
            table
                .ulid_fk(UserEntity::table_name(), true)
                .set_is_unique(true);
        })
        .await;
}
