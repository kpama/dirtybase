use dirtybase_contract::app_contract::Context;
use dirtybase_contract::auth_contract::{Actor, AuthUserStatus};
use dirtybase_contract::db_contract::TableModel;
use dirtybase_contract::db_contract::base::manager::Manager;
use dirtybase_contract::db_contract::migration::Migration;

pub struct Mig1740151519CreateAuthUserTable;

#[dirtybase_contract::async_trait]
impl Migration for Mig1740151519CreateAuthUserTable {
    async fn up(&self, manager: &Manager, _: &Context) -> Result<(), anyhow::Error> {
        manager
            .create_table_schema(Actor::table_name(), |table| {
                table.uuid_as_id(None);
                table
                    .string(Actor::col_name_for_username())
                    .set_is_unique(true);
                table
                    .string(Actor::col_name_for_password())
                    .set_is_nullable(true);
                table
                    .boolean(Actor::col_name_for_reset_password())
                    .default_is_false(); // A flag that indicates a password change is required
                table
                    .string(Actor::col_name_for_status())
                    .set_default_from(AuthUserStatus::Pending);
                table
                    .sized_string(Actor::col_name_for_salt(), 100)
                    .set_is_nullable(false);
                table
                    .number(Actor::col_name_for_login_attempt())
                    .set_default_from(0);
                table
                    .datetime(Actor::col_name_for_last_login_at())
                    .set_is_nullable(true);
                table
                    .string(Actor::col_name_for_email_hash())
                    .set_is_unique(true);
                table
                    .datetime(Actor::col_name_for_verified_at())
                    .set_is_nullable(true);
                table.timestamps();
                table.soft_deletable();
            })
            .await
    }

    #[cfg(not(feature = "permission"))]
    async fn down(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        manager.drop_table(Actor::table_name()).await
    }

    #[cfg(feature = "permission")]
    async fn down(&self, _manager: &Manager, _: &Context) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
