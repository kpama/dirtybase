use dirtybase_common::db::TableModel;
use dirtybase_contract::db_contract::base::manager::Manager;
use dirtybase_contract::db_contract::migration::Migration;

use crate::CacheEntry;
use crate::model::cache_db_store::CacheDbStoreEntity;

pub struct Mig1744604919CreateCacheTables;

#[dirtybase_contract::async_trait]
impl Migration for Mig1744604919CreateCacheTables {
    async fn up(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        manager
            .create_table_schema(CacheDbStoreEntity::table_name(), |bp| {
                bp.id(None);
                bp.string(CacheEntry::col_name_for_key())
                    .set_is_nullable(false)
                    .set_is_unique(true);
                bp.text(CacheEntry::col_name_for_value())
                    .set_is_nullable(true);
                bp.integer(CacheEntry::col_name_for_expiration())
                    .set_is_nullable(true);
                bp.timestamps();

                bp.index(&[CacheEntry::col_name_for_expiration()]);
            })
            .await
    }

    async fn down(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        manager.drop_table(CacheDbStoreEntity::table_name()).await
    }
}
