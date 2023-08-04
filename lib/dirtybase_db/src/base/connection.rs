use std::fmt::Debug;

use super::schema::{DatabaseKind, SchemaManagerTrait};
use async_trait::async_trait;

#[async_trait]
pub trait ConnectionPoolRegisterTrait: Send {
    async fn register(&self, conn_str: &str, max: u32) -> Option<Box<dyn ConnectionPoolTrait>>;
}

#[async_trait]
pub trait ConnectionPoolTrait: Debug + Send + Sync {
    /// Calls by the ConnectionManagerCollection
    fn schema_manger(&self) -> Box<dyn SchemaManagerTrait + Send + Sync>;
    fn id(&self) -> DatabaseKind;
}
