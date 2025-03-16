use dirtybase_contract::db::TableEntityTrait;
use dirtybase_contract::db::base::manager::Manager;
use dirtybase_contract::db::migration::Migration;
use dirtybase_contract::user::status::UserStatus;

use crate::UserEntity;

pub struct Mig0000000000CreateUserTable;

#[dirtybase_contract::async_trait]
impl Migration for Mig0000000000CreateUserTable {
    async fn up(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        manager
            .create_table_schema(UserEntity::table_name(), |table| {
                table.uuid_as_id(None);
                table.string("username").set_is_unique(true);
                table.string("email").set_is_unique(true);
                table.string("password").set_is_nullable(true);
                table.boolean("reset_password").set_default_from(false); // A flag that indicates a password change is required
                table.string("status").set_default_from(UserStatus::Pending);
                table.boolean("is_sys_admin").set_default_from(false); // A flag that indicates if this user is an admin
                table.sized_string("salt", 512).set_is_nullable(false);
                table.number("login_attempt").set_default_from(0);
                table.datetime("last_login_at").set_is_nullable(true);
                table.timestamps();
                table.soft_deletable();
            })
            .await;
        Ok(())
    }

    async fn down(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        manager.drop_table(UserEntity::table_name()).await;
        Ok(())
    }
}
