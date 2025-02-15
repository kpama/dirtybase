use std::fmt::Debug;

use super::schema::{DatabaseKind, SchemaManagerTrait};
use async_trait::async_trait;

#[async_trait]
pub trait ConnectionPoolTrait: Debug + Send + Sync {
    /// Calls by the ConnectionManagerCollection
    fn schema_manger(&self) -> Box<dyn SchemaManagerTrait + Send + Sync>;
    async fn close(&self);
    fn id(&self) -> DatabaseKind;
}
