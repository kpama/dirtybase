use std::{collections::HashMap, fmt::Debug};

use crate::config::DirtybaseDbConfig;

use super::schema::{ClientType, DatabaseKind, SchemaManagerTrait};
use async_trait::async_trait;

#[async_trait]
pub trait ConnectionPoolRegisterTrait: Send {
    async fn register(
        &self,
        config: &DirtybaseDbConfig,
    ) -> Option<HashMap<ClientType, Box<dyn ConnectionPoolTrait>>>;
}

#[async_trait]
pub trait ConnectionPoolTrait: Debug + Send + Sync {
    /// Calls by the ConnectionManagerCollection
    fn schema_manger(&self) -> Box<dyn SchemaManagerTrait + Send + Sync>;
    fn id(&self) -> DatabaseKind;
}
