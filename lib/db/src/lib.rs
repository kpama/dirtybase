mod column_value_builder;
mod pool_manager;
mod table_entity;

pub mod base;
pub mod config;
pub mod connector;
pub mod event;
pub mod event_handler;
pub mod field_values;
pub mod migration;
pub mod query_values;
pub mod types;

use orsomafo::Dispatchable;
use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

use base::schema::DatabaseKind;
use config::DirtybaseDbConfig;
use event::SchemeWroteEvent;
use event_handler::HandleSchemaWroteEvent;

pub use anyhow;
pub use column_value_builder::*;
pub use dirtybase_config;
pub use pool_manager::*;
pub use table_entity::*;

pub(crate) static LAST_WRITE_TS: OnceLock<RwLock<HashMap<DatabaseKind, i64>>> = OnceLock::new();

pub const USER_TABLE: &str = "core_user";

pub async fn setup(config: &dirtybase_config::DirtyConfig) -> ConnectionPoolManager {
    let base_config = DirtybaseDbConfig::new(config).await;

    LAST_WRITE_TS.get_or_init(|| RwLock::new(HashMap::new()));

    // event handlers
    _ = SchemeWroteEvent::subscribe::<HandleSchemaWroteEvent>().await;

    setup_using(base_config).await
}

pub async fn setup_using(config: DirtybaseDbConfig) -> ConnectionPoolManager {
    let pool_manager = ConnectionPoolManager::new(config).await;

    // busybody::helpers::service_container().set_type(pool_manager.clone());

    pool_manager
}

#[busybody::async_trait]
impl busybody::Injectable for ConnectionPoolManager {
    async fn inject(container: &busybody::ServiceContainer) -> Self {
        container.get_type().unwrap()
    }
}
