use anyhow::Ok;
use dirtybase_contract::db_contract::TableEntityTrait;
use dirtybase_contract::db_contract::base::manager::Manager;
use dirtybase_contract::db_contract::migration::Migration;

use crate::storage::database::SessionTable;

pub struct Mig1744202277CreateSessionTable;

#[dirtybase_contract::async_trait]
impl Migration for Mig1744202277CreateSessionTable {
    async fn up(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        manager
            .create_table_schema(SessionTable::table_name(), |table| {
                table
                    .string(SessionTable::col_name_for_id())
                    .set_is_unique(true);
                table
                    .json(SessionTable::col_name_for_data())
                    .default_is_empty_object();
                table.created_at();
                table
                    .integer(SessionTable::col_name_for_expires())
                    .set_is_nullable(true);
            })
            .await
    }

    async fn down(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        manager.drop_table(SessionTable::table_name()).await;
        Ok(())
    }
}
