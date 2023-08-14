use dirtybase_db::base::manager::Manager;
use dirtybase_db_types::TableEntityTrait;

mod cache_db_store_entity;
mod cache_db_store_repository;

pub use cache_db_store_entity::CacheDbStoreEntity;
pub use cache_db_store_repository::CacheDbStoreRepository;

pub async fn setup_cache_db_store_table(manager: &Manager) {
    manager
        .create_table_schema(CacheDbStoreEntity::table_name(), |table| {
            table
                .string(CacheDbStoreEntity::id_column().unwrap())
                .set_is_unique(true);
            table
                .json(CacheDbStoreEntity::col_name_for_value())
                .set_is_nullable(true);
            table
                .integer(CacheDbStoreEntity::col_name_for_expiration())
                .set_is_nullable(true);
        })
        .await;
}
