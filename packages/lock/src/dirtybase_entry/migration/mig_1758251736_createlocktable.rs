use dirtybase_contract::anyhow;
use dirtybase_contract::db_contract::base::manager::Manager;
use dirtybase_contract::db_contract::migration::Migration;

use crate::storage::LockDataWrapper;
use dirtybase_contract::db_contract::TableModel;

pub struct Mig1758251736CreateLockTable;

#[dirtybase_contract::async_trait]
impl Migration for Mig1758251736CreateLockTable {
    async fn up(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        manager
            .create_table_schema(LockDataWrapper::table_name(), |bp| {
                //
                bp.id(Some(LockDataWrapper::col_name_for_id()));
                bp.string("key").set_is_unique(true).set_is_nullable(false);
                bp.string("owner").set_is_nullable(false);
                bp.integer("expires").set_is_nullable(false).set_default(5);
            })
            .await
    }

    async fn down(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        manager.drop_table(LockDataWrapper::table_name()).await
    }
}
