use async_trait::async_trait;
use sqlx::{any::AnyKind, MySql, Pool};
use std::sync::Arc;

use super::{query::QueryBuilder, table::BaseTable};

#[async_trait]
pub trait SchemaManagerTrait {
    fn instance(db_pool: Arc<Pool<MySql>>) -> Self
    where
        Self: Sized;

    fn kind(&self) -> AnyKind;

    // update an existing table
    fn fetch_table_for_update(&self, name: &str) -> BaseTable;

    // commit schema changes
    async fn commit(&self, table: BaseTable);

    // build a select query
    fn query(&mut self, query_builder: QueryBuilder) -> &dyn SchemaManagerTrait;

    async fn save(&self, query_builder: QueryBuilder);

    async fn fetch_all_as_json(&self) -> Vec<serde_json::Value>;

    // checks if a table exist in the database
    async fn has_table(&self, name: &str) -> bool;
}
