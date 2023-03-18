pub struct User {
    // internal_id: u64,
    // id: String
    // username: String
    // password: String
}
// Core User table name
pub static USER_TABLE: &str = "_core_users";

pub fn user_table_name() -> String {
    USER_TABLE.to_owned()
}

// We need to have this table in the orm lib as
// the "own" fields are assuming there is a user
// table
pub async fn setup_users_table(manager: &super::manager::Manager) {
    if !manager.has_table(USER_TABLE).await {
        manager
            .create(USER_TABLE, |table| {
                table.id_set();
                table.string("username");
                table.string("email");
                // password, open ID, third party, magic link?
                // the user could be using openID to login !?!?!
                table.timestamps();
                table.soft_deletable();
            })
            .await;
    }
}
