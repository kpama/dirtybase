use super::{query::QueryBuilder, table::BaseTable};
use async_trait::async_trait;
use dirtybase_db_types::types::{ColumnAndValue, FromColumnAndValue, StructuredColumnAndValue};
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
pub trait SchemaManagerTrait: Send + Sync {
    // update an existing table
    fn fetch_table_for_update(&self, name: &str) -> BaseTable;

    // commit schema changes
    async fn commit(&self, table: BaseTable);

    async fn execute(&self, query_builder: QueryBuilder);

    async fn fetch_all(
        &self,
        query_builder: &QueryBuilder,
    ) -> Result<Vec<ColumnAndValue>, anyhow::Error>;

    async fn fetch_one(
        &self,
        query_builder: &QueryBuilder,
    ) -> Result<ColumnAndValue, anyhow::Error>;

    async fn fetch_one_to<T: FromColumnAndValue>(
        &self,
        query_builder: &QueryBuilder,
    ) -> Result<T, anyhow::Error>
    where
        Self: Sized,
    {
        let result = self.fetch_one(query_builder).await;

        if let Ok(row) = result {
            Ok(T::from_column_value(row))
        } else {
            Err(result.err().unwrap())
        }
    }

    async fn fetch_all_to<T>(&self, query: &QueryBuilder) -> Result<Vec<T>, anyhow::Error>
    where
        Self: Sized,
        T: FromColumnAndValue,
    {
        let result = self.fetch_all(query).await;
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

pub struct SchemaWrapper<'a> {
    pub(crate) query_builder: QueryBuilder,
    pub(crate) inner: &'a dyn SchemaManagerTrait,
}

impl<'a> SchemaWrapper<'a> {
    pub async fn fetch_all(&self) -> Result<Vec<StructuredColumnAndValue>, anyhow::Error> {
        let results = self.inner.fetch_all(&self.query_builder).await;
        if let Ok(r) = results {
            Ok(StructuredColumnAndValue::from_results(r))
        } else {
            Err(results.err().unwrap())
        }
    }

    pub async fn fetch_one(&self) -> Result<StructuredColumnAndValue, anyhow::Error> {
        let result = self.inner.fetch_one(&self.query_builder).await;

        if let Ok(r) = result {
            Ok(StructuredColumnAndValue::from_a_result(r))
        } else {
            Err(result.err().unwrap())
        }
    }

    pub async fn fetch_one_to<T: FromColumnAndValue>(&self) -> Result<T, anyhow::Error>
    where
        Self: Sized,
    {
        let result = self.fetch_one().await;

        if let Ok(row) = result {
            Ok(T::from_column_value(row.fields()))
        } else {
            Err(result.err().unwrap())
        }
    }

    pub async fn fetch_all_to<T>(&self) -> Result<Vec<T>, anyhow::Error>
    where
        Self: Sized,
        T: FromColumnAndValue,
    {
        let result = self.fetch_all().await;
        if let Ok(records) = result {
            Ok(records
                .into_iter()
                .map(|row| T::from_column_value(row.fields()))
                .collect::<Vec<T>>())
        } else {
            Err(result.err().unwrap())
        }
    }
}
