mod dirtybase_user_entity;
mod dirtybase_user_repository;
mod dirtybase_user_service;

pub mod dirtybase_user_helpers;
pub mod in_dtos;
pub mod out_dtos;

use dirtybase_db::base::manager::Manager;
use dirtybase_db::base::table::ID_FIELD;
use dirtybase_db::base::table::INTERNAL_ID_FIELD;
use dirtybase_db::entity::user::USER_TABLE;

pub use dirtybase_user_service::DirtybaseUserService;

// Dirtybase user table
pub const DIRTYBASE_USER_TABLE: &str = "core_dirtybase_user";

// Fields
pub const DIRTYBASE_USER_TABLE_INTERNAL_ID_FIELD: &str = INTERNAL_ID_FIELD;
pub const DIRTYBASE_USER_TABLE_ID_FIELD: &str = ID_FIELD;
pub const DIRTYBASE_USER_TABLE_LOGIN_ATTEMPT_FIELD: &str = "login_attemp";
pub const DIRTYBASE_USER_TABLE_LAST_LOGIN_FIELD: &str = "last_login_at";
pub const DIRTYBASE_USER_TABLE_CORE_USER_FIELD: &str = "core_user_id";

pub async fn setup_dirtybase_user_table(manager: &Manager) {
    if !manager.has_table(DIRTYBASE_USER_TABLE).await {
        manager
            .create_table_schema(DIRTYBASE_USER_TABLE, |table| {
                // Relation to the "core_user" table
                table.ulid_fk(USER_TABLE, true).set_is_unique(true);

                // The number of times the user tried to login
                table
                    .integer(DIRTYBASE_USER_TABLE_LOGIN_ATTEMPT_FIELD)
                    .set_default("0");

                // Last time the user successfully logged in
                table
                    .date(DIRTYBASE_USER_TABLE_LAST_LOGIN_FIELD)
                    .set_is_nullable(true);
            })
            .await
    }
}
