use crate::base::{
    manager::Manager,
    table::{CREATED_AT_FIELD, DELETED_AT_FIELD, ID_FIELD, INTERNAL_ID_FIELD, UPDATED_AT_FIELD},
};

mod user_entity;
mod user_helpers;
mod user_repository;
mod user_service;

pub mod dtos;

pub use user_entity::UserEntity;
pub use user_helpers::*;
pub use user_repository::UserRepository;
pub use user_service::UserService;

// Core User table name
pub const USER_TABLE: &str = "core_user";

// Fields
pub const USER_TABLE_USERNAME_FIELD: &str = "username";
pub const USER_TABLE_EMAIL_FIELD: &str = "email";
pub const USER_TABLE_PASSWORD_FIELD: &str = "password";
pub const USER_TABLE_RESET_PASSWORD_FIELD: &str = "reset_password";
pub const USER_TABLE_STATUS_FIELD: &str = "status";
pub const USER_TABLE_INTERNAL_ID_FIELD: &str = INTERNAL_ID_FIELD;
pub const USER_TABLE_ID_FIELD: &str = ID_FIELD;
pub const USER_TABLE_CREATED_AT_FIELD: &str = CREATED_AT_FIELD;
pub const USER_TABLE_UPDATED_AT_FIELD: &str = UPDATED_AT_FIELD;
pub const USER_TABLE_DELETED_AT_FIELD: &str = DELETED_AT_FIELD;

pub fn hash_password(raw: &str) -> String {
    bcrypt::hash(raw, 8).unwrap()
}

pub fn verify_password(raw: &str, hash: &str) -> bool {
    bcrypt::verify(raw, hash).unwrap()
}

// We need to have this table in the orm lib as
// the "own" fields are assuming there is a user
// table
pub async fn setup_users_table(manager: &Manager) {
    if !manager.has_table(USER_TABLE).await {
        manager
            .create_table_schema(USER_TABLE, |table| {
                table.id_set();
                table.string(USER_TABLE_USERNAME_FIELD).set_is_unique(true);
                table.string(USER_TABLE_EMAIL_FIELD).set_is_unique(true);
                table
                    .string(USER_TABLE_PASSWORD_FIELD)
                    .set_is_nullable(true);
                table
                    .boolean(USER_TABLE_RESET_PASSWORD_FIELD)
                    .set_default_from(false); // A flag that indicates a password change is required
                table
                    .string(USER_TABLE_STATUS_FIELD)
                    .set_default_from(UserStatus::Pending);
                table.timestamps();
                table.soft_deletable();
            })
            .await;
    }
}
