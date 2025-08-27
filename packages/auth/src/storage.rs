pub mod database_storage;
pub mod memory_storage;

pub use dirtybase_contract::fama::PipeContent;
pub use dirtybase_contract::fama::PipelineBuilderTrait;

use crate::database_storage::AuthUserDatabaseStorage;
use crate::memory_storage::AuthUserMemoryStorage;

pub(crate) async fn register_storages() {
    // database storage
    AuthUserDatabaseStorage::register().await;

    // memory storage
    AuthUserMemoryStorage::register().await;
}
