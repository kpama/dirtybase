mod dirtybase_user_entity;
mod dirtybase_user_repository;
mod dirtybase_user_service;

pub mod dirtybase_user_cache;
pub mod dirtybase_user_helpers;
pub mod dtos;
pub mod event;

use dirtybase_db::base::manager::Manager;
use dirtybase_db::base::table::ID_FIELD;
use dirtybase_db::base::table::INTERNAL_ID_FIELD;

use dirtybase_contract::db::entity::user::UserEntity;
use dirtybase_db::TableEntityTrait;

pub use dirtybase_user_entity::DirtybaseUserEntity;
pub use dirtybase_user_repository::DirtybaseUserRepository;
pub use dirtybase_user_service::DirtybaseUserService;

// Dirtybase user table
pub const DIRTYBASE_USER_TABLE: &str = "core_dirtybase_user";

// Fields
pub const DIRTYBASE_USER_TABLE_INTERNAL_ID_FIELD: &str = INTERNAL_ID_FIELD;
pub const DIRTYBASE_USER_TABLE_ID_FIELD: &str = ID_FIELD;
pub const DIRTYBASE_USER_TABLE_LOGIN_ATTEMPT_FIELD: &str = "login_attempt";
pub const DIRTYBASE_USER_TABLE_LAST_LOGIN_FIELD: &str = "last_login_at";
pub const DIRTYBASE_USER_TABLE_CORE_USER_FIELD: &str = "core_user_id";
pub const DIRTYBASE_USER_TABLE_CORE_SALT_FIELD: &str = "salt";

pub async fn setup_dirtybase_user_table(manager: &Manager) {
    manager
        .create_table_schema(DirtybaseUserEntity::table_name(), |table| {
            // Relation to the "core_user" table
            table
                .ulid_fk(UserEntity::table_name(), true)
                .set_is_unique(true);

            // The number of times the user tried to login
            table
                .integer(DirtybaseUserEntity::col_name_for_login_attempt())
                .set_default("0");

            // Last time the user successfully logged in
            table
                .timestamp(DirtybaseUserEntity::col_name_for_last_login_at())
                .set_is_nullable(true);

            // The salt column. Used for salting tokens generated for this user
            table
                .sized_string(DirtybaseUserEntity::col_name_for_salt(), 512)
                .set_is_nullable(false);
        })
        .await
}
