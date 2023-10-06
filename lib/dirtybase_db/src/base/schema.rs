use super::{query::QueryBuilder, table::BaseTable};
use async_trait::async_trait;
use dirtybase_db_types::types::{ColumnAndValue, FromColumnAndValue, StructuredColumnAndValue};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, serde::Deserialize)]
pub enum DatabaseKind {
    #[serde(rename(deserialize = "mysql"))]
    Mysql,
    #[serde(rename(deserialize = "sqlite"))]
    Sqlite,
    #[serde(rename(deserialize = "postgres"))]
    Postgres,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum ClientType {
    Read,
    Write,
}

impl Default for DatabaseKind {
    fn default() -> Self {
        Self::Mysql
    }
}

impl From<&str> for DatabaseKind {
    fn from(value: &str) -> Self {
        match value.to_lowercase() {
            _ if value.starts_with("mysql:")
                || value.starts_with("mariadb:")
                || value == "mysql"
                || value == "mariadb" =>
            {
                Self::Mysql
            }
            _ if value.starts_with("sqlite:") || value == "sqlite" => Self::Sqlite,
            _ if value.starts_with("postgres:") || value == "postgres" => Self::Postgres,
            _ => panic!("Unknown database type"),
        }
    }
}

impl From<DatabaseKind> for String {
    fn from(value: DatabaseKind) -> Self {
        String::from(&value)
    }
}

impl From<&DatabaseKind> for String {
    fn from(value: &DatabaseKind) -> Self {
        match value {
            DatabaseKind::Mysql => "mysql".to_string(),
            DatabaseKind::Sqlite => "sqlite".to_string(),
            DatabaseKind::Postgres => "postgres".to_string(),
        }
    }
}

#[async_trait]
pub trait RelationalDbTrait: SchemaManagerTrait {
    fn kind(&self) -> DatabaseKind;
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

pub struct SchemaWrapper {
    pub(crate) query_builder: QueryBuilder,
    pub(crate) inner: Box<dyn SchemaManagerTrait>,
}

impl SchemaWrapper {
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
