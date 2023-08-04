mod dirtybase_user_entity;
mod dirtybase_user_repository;
mod dirtybase_user_service;

pub mod dirtybase_user_helpers;
pub mod dtos;

use dirtybase_db::base::manager::Manager;
use dirtybase_db::base::table::ID_FIELD;
use dirtybase_db::base::table::INTERNAL_ID_FIELD;

use dirtybase_db::entity::user::UserEntity;
use dirtybase_db_types::TableEntityTrait;

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
    if !manager.has_table(DIRTYBASE_USER_TABLE).await {
        manager
            .create_table_schema(DIRTYBASE_USER_TABLE, |table| {
                // Relation to the "core_user" table
                table
                    .ulid_fk(UserEntity::table_name(), true)
                    .set_is_unique(true);

                // The number of times the user tried to login
                table
                    .integer(DIRTYBASE_USER_TABLE_LOGIN_ATTEMPT_FIELD)
                    .set_default("0");

                // Last time the user successfully logged in
                table
                    .date(DIRTYBASE_USER_TABLE_LAST_LOGIN_FIELD)
                    .set_is_nullable(true);

                // The salt column. Used for salting tokens generated for this user
                table
                    .sized_string(DIRTYBASE_USER_TABLE_CORE_SALT_FIELD, 512)
                    .set_is_nullable(false);
            })
            .await
    }
}
