use dirtybase_contract::db::base::manager::Manager;
use dirtybase_contract::db::TableEntityTrait;

mod cache_db_store_entity;
mod cache_db_store_repository;
mod cache_db_tag_store_entity;

pub use self::cache_db_tag_store_entity::CacheDbPivotEntity;
pub use self::cache_db_tag_store_entity::CacheDbTagStoreEntity;
pub use cache_db_store_entity::CacheDbStoreEntity;
pub use cache_db_store_repository::CacheDbStoreRepository;

pub async fn setup_cache_db_store_tables(manager: &Manager) {
    setup_cache_db_store_table(manager).await;
    setup_catch_db_tag_store_table(manager).await;
    setup_catch_db_tag_entries_pivot(manager).await;
}

// Main cache store table
async fn setup_cache_db_store_table(manager: &Manager) {
    manager
        .create_table_schema(CacheDbStoreEntity::table_name(), |table| {
            table
                .string(CacheDbStoreEntity::id_column().unwrap())
                .set_is_unique(true);
            table
                .text(CacheDbStoreEntity::col_name_for_content())
                .set_is_nullable(true);
            table
                .integer(CacheDbStoreEntity::col_name_for_expiration())
                .set_is_nullable(true);
        })
        .await;
}

// Tags table
async fn setup_catch_db_tag_store_table(manager: &Manager) {
    manager
        .create_table_schema(CacheDbTagStoreEntity::table_name(), |table| {
            // table.id(CacheDbTagStoreEntity::id_column());
            table
                .string(CacheDbTagStoreEntity::col_name_for_tag())
                .set_is_nullable(false)
                .set_is_unique(true);
        })
        .await;
}

// Tags to entry pivot table
async fn setup_catch_db_tag_entries_pivot(manager: &Manager) {
    manager
        .create_table_schema(CacheDbPivotEntity::table_name(), |table| {
            table
                .string(CacheDbPivotEntity::col_name_for_core_cache_tags_id())
                .set_is_nullable(false)
                .references(
                    CacheDbTagStoreEntity::table_name(),
                    CacheDbTagStoreEntity::col_name_for_tag(),
                    true,
                );
            table
                .string(CacheDbPivotEntity::col_name_for_core_cache_key())
                .set_is_nullable(false)
                .references(
                    CacheDbStoreEntity::table_name(),
                    CacheDbStoreEntity::col_name_for_key(),
                    true,
                );

            table.primary_index(&[
                CacheDbPivotEntity::col_name_for_core_cache_tags_id(),
                CacheDbPivotEntity::col_name_for_core_cache_key(),
            ]);
        })
        .await;
}
