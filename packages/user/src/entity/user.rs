use dirtybase_contract::db::base::manager::Manager;

mod user_entity;
mod user_repository;
mod user_service;

pub mod dtos;

use dirtybase_contract::db::TableEntityTrait;
use dirtybase_contract::user::status::UserStatus;
pub use user_entity::UserEntity;
pub use user_repository::UserRepository;
pub use user_service::UserService;

pub fn hash_password(raw: &str) -> String {
    bcrypt::hash(raw, 8).unwrap()
}

pub fn verify_password(raw: &str, hash: &str) -> bool {
    bcrypt::verify(raw, hash).unwrap()
}

pub async fn setup_users_table(manager: &Manager) {
    manager
        .create_table_schema(UserEntity::table_name(), |table| {
            table.id_set();
            table
                .string(UserEntity::col_name_for_username())
                .set_is_unique(true);
            table
                .string(UserEntity::col_name_for_email())
                .set_is_unique(true);
            table
                .string(UserEntity::col_name_for_password())
                .set_is_nullable(true);
            table
                .boolean(UserEntity::col_name_for_reset_password())
                .set_default_from(false); // A flag that indicates a password change is required
            table
                .string(UserEntity::col_name_for_status())
                .set_default_from(UserStatus::Pending);
            table
                .boolean(UserEntity::col_name_for_is_sys_admin())
                .set_default_from(false); // A flag that indicates if this user is an admin
            table
                .sized_string(UserEntity::col_name_for_salt(), 512)
                .set_is_nullable(false);
            table
                .number(UserEntity::col_name_for_login_attempt())
                .set_default_from(0);
            table
                .datetime(UserEntity::col_name_for_last_login_at())
                .set_is_nullable(true);
            table.timestamps();
            table.soft_deletable();
        })
        .await;
}
