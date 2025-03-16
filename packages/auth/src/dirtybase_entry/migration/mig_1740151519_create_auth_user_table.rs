use anyhow::anyhow;
use dirtybase_contract::auth::AuthUserStatus;
use dirtybase_contract::db::base::manager::Manager;
use dirtybase_contract::db::migration::Migration;

use crate::AUTH_USER_TABLE;

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
                table.boolean("is_sys_admin").set_default_from(false); // A flag that indicates if this user is an admin
                table.sized_string("salt", 100).set_is_nullable(false);
                table.number("login_attempt").set_default_from(0);
                table.datetime("last_login_at").set_is_nullable(true);
                table.string("email_hash").set_is_unique(true);
                table.timestamps();
                table.soft_deletable();
            })
            .await
    }

    async fn down(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        if manager.drop_table(AUTH_USER_TABLE).await {
            return Ok(());
        }
        Err(anyhow!("could not drop: {}", AUTH_USER_TABLE))
    }
}
