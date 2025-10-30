use dirtybase_contract::auth_contract::AuthUserStatus;
use dirtybase_contract::db_contract::base::manager::Manager;
use dirtybase_contract::db_contract::migration::Migration;

use crate::storage::database_storage::AUTH_USER_TABLE;

pub struct Mig1740151519CreateAuthUserTable;

#[dirtybase_contract::async_trait]
impl Migration for Mig1740151519CreateAuthUserTable {
    async fn up(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        manager
            .create_table_schema(AUTH_USER_TABLE, |table| {
                table.uuid_as_id(None);
                table.string("username").set_is_unique(true);
                table.string("password").set_is_nullable(true);
                table.boolean("reset_password").set_default_from(false); // A flag that indicates a password change is required
                table
                    .string("status")
                    .set_default_from(AuthUserStatus::Pending);
                table.sized_string("salt", 100).set_is_nullable(false);
                table.number("login_attempt").set_default_from(0);
                table.datetime("last_login_at").set_is_nullable(true);
                table.string("email_hash").set_is_unique(true);
                table.datetime("verified_at").set_is_nullable(true);
                table.timestamps();
                table.soft_deletable();
            })
            .await
    }

    async fn down(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        manager.drop_table(AUTH_USER_TABLE).await
    }
}
