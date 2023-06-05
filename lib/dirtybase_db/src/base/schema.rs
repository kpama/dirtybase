use super::{
    query::QueryBuilder,
    table::BaseTable,
    types::{ColumnAndValue, FromColumnAndValue},
};
use async_trait::async_trait;
use sqlx::any::AnyKind;
use std::sync::Arc;

#[async_trait]
pub trait RelationalDbTrait: SchemaManagerTrait {
    fn kind(&self) -> AnyKind;
}

pub trait GraphDbTrait<T> {
    fn instance(client: Arc<T>) -> Self
    where
        Self: Sized;
}

#[async_trait]
pub trait SchemaManagerTrait {
    // update an existing table
    fn fetch_table_for_update(&self, name: &str) -> BaseTable;

    // commit schema changes
    async fn commit(&self, table: BaseTable);

    // build a select query
    fn query(&mut self, query_builder: QueryBuilder) -> &dyn SchemaManagerTrait;

    async fn execute(&self, query_builder: QueryBuilder);

    async fn fetch_all_as_json(&self) -> Result<Vec<serde_json::Value>, anyhow::Error>;

    async fn fetch_one_as_json(&self) -> Result<serde_json::Value, anyhow::Error>;

    async fn fetch_all_as_field_value(&self) -> Result<Vec<ColumnAndValue>, anyhow::Error>;

    async fn fetch_one_as_field_value(&self) -> Result<ColumnAndValue, anyhow::Error>;

    async fn fetch_one<T: FromColumnAndValue>(&self) -> Result<T, anyhow::Error>
    where
        Self: Sized,
    {
        let result = self.fetch_one_as_field_value().await;

        if let Ok(row) = result {
            Ok(T::from_column_value(row))
        } else {
            Err(result.err().unwrap())
        }
    }

    async fn fetch_one_all<T: FromColumnAndValue>(&self) -> Result<Vec<T>, anyhow::Error>
    where
        Self: Sized,
    {
        let result = self.fetch_all_as_field_value().await;
        if let Ok(records) = result {
            Ok(records
                .into_iter()
                .map(T::from_column_value)
                .collect::<Vec<T>>())
        } else {
            Err(result.err().unwrap())
        }
    }

    // checks if a table exist in the database
    async fn has_table(&self, name: &str) -> bool;
}
