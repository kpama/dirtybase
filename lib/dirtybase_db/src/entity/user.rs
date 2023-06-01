use crate::base::manager::Manager;

mod user_entity;
mod user_service;

pub use user_entity::UserEntity;

// Core User table name
pub static USER_TABLE: &str = "core_user";

pub fn hash_password(raw: &str) -> String {
    bcrypt::hash(raw, 8).unwrap()
}

pub fn verify_password(raw: &str, hash: &str) {
    bcrypt::verify(raw, hash).unwrap();
}

// We need to have this table in the orm lib as
// the "own" fields are assuming there is a user
// table
pub async fn setup_users_table(manager: &Manager) {
    if !manager.has_table(USER_TABLE).await {
        manager
            .create_table_schema(USER_TABLE, |table| {
                table.id_set();
                table.string("username").set_is_unique(true);
                table.string("email").set_is_unique(true);
                table.string("password").set_is_nullable(true);
                // password, open ID, third party, magic link?
                // the user could be using openID to login !?!?!
                table.timestamps();
                table.soft_deletable();
            })
            .await;
    }
}
