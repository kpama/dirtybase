use super::{
    query::{QueryAction, QueryBuilder},
    table::TableBlueprint,
};
use crate::db_contract::{
    field_values::FieldValue,
    types::{ColumnAndValue, FromColumnAndValue, StructuredColumnAndValue},
};
use anyhow::Result;
use async_trait::async_trait;
use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Clone,
    serde::Deserialize,
    serde::Serialize,
    Default,
)]
pub struct DatabaseKind(Arc<String>);

impl Display for DatabaseKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for DatabaseKind {
    fn from(value: &str) -> Self {
        Self(Arc::new(value.to_string()))
    }
}

impl From<DatabaseKind> for String {
    fn from(value: DatabaseKind) -> Self {
        String::from(value.0.as_str())
    }
}

impl From<&DatabaseKind> for String {
    fn from(value: &DatabaseKind) -> Self {
        value.0.as_str().to_string()
    }
}

impl DatabaseKind {
    pub fn as_str(&self) -> &str {
        &self.0.as_str()
    }
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Clone,
    Copy,
    serde::Deserialize,
    serde::Serialize,
    Default,
)]
pub enum ClientType {
    #[serde(alias = "read")]
    Read,
    #[default]
    #[serde(alias = "write")]
    Write,
}

#[async_trait]
pub trait RelationalDbTrait: SchemaManagerTrait {
    fn kind(&self) -> DatabaseKind;
}

#[async_trait]
pub trait SchemaManagerTrait: Send + Sync {
    // update an existing table
    fn fetch_table_for_update(&self, name: &str) -> TableBlueprint;

    // commit schema changes
    async fn apply(&self, table: TableBlueprint) -> Result<()>;

    async fn execute(&self, query_builder: QueryBuilder) -> Result<()>;

    async fn fetch_all(
        &self,
        query_builder: &QueryBuilder,
    ) -> Result<Option<Vec<ColumnAndValue>>, anyhow::Error>;

    async fn stream_result(
        &self,
        query_builder: &QueryBuilder,
        sender: tokio::sync::mpsc::Sender<ColumnAndValue>,
    ) -> Result<()>;

    async fn fetch_one(
        &self,
        query_builder: &QueryBuilder,
    ) -> Result<Option<ColumnAndValue>, anyhow::Error>;

    async fn fetch_one_to<T: FromColumnAndValue>(
        &self,
        query_builder: &QueryBuilder,
    ) -> Result<Option<T>, anyhow::Error>
    where
        Self: Sized,
    {
        let result = self.fetch_one(query_builder).await;

        if let Ok(row) = result {
            match row {
                Some(r) => Ok(Some(T::from_column_value(r)?)),
                None => Ok(None),
            }
        } else {
            Err(result.err().unwrap())
        }
    }

    async fn fetch_all_to<T>(&self, query: &QueryBuilder) -> Result<Option<Vec<T>>, anyhow::Error>
    where
        Self: Sized,
        T: FromColumnAndValue,
    {
        let result = self.fetch_all(query).await;
        if let Ok(records) = result {
            match records {
                Some(rs) => Ok(Some(
                    rs.into_iter()
                        .flat_map(T::from_column_value)
                        .collect::<Vec<T>>(),
                )),
                None => Ok(None),
            }
        } else {
            Err(result.err().unwrap())
        }
    }

    // Checks if a table exist in the database
    async fn has_table(&self, name: &str) -> Result<bool, anyhow::Error>;

    async fn drop_table(&self, name: &str) -> Result<()>;

    async fn rename_table(&self, old: &str, new: &str) -> Result<()> {
        self.execute(QueryBuilder::new(
            old,
            QueryAction::RenameTable(new.to_string()),
        ))
        .await
    }

    async fn drop_column(&self, table: &str, column: &str) -> Result<()> {
        self.execute(QueryBuilder::new(
            table,
            QueryAction::DropColumn(column.to_string()),
        ))
        .await
    }

    async fn rename_column(&self, table: &str, old: &str, new: &str) -> Result<()> {
        self.execute(QueryBuilder::new(
            table,
            QueryAction::RenameColumn {
                old: old.to_string(),
                new: new.to_string(),
            },
        ))
        .await
    }

    async fn raw_insert(&self, sql: &str, values: Vec<FieldValue>) -> Result<bool, anyhow::Error>;

    async fn raw_update(&self, sql: &str, params: Vec<FieldValue>) -> Result<u64, anyhow::Error>;

    async fn raw_delete(&self, sql: &str, values: Vec<FieldValue>) -> Result<u64, anyhow::Error>;

    async fn raw_select(
        &self,
        sql: &str,
        params: Vec<FieldValue>,
    ) -> Result<Vec<ColumnAndValue>, anyhow::Error>;

    async fn raw_statement(&self, sql: &str) -> Result<bool, anyhow::Error>;
}

pub struct SchemaWrapper {
    pub(crate) query_builder: QueryBuilder,
    pub(crate) inner: Box<dyn SchemaManagerTrait>,
}

impl SchemaWrapper {
    pub fn new(qb: QueryBuilder, schema_manager: Box<dyn SchemaManagerTrait>) -> Self {
        Self {
            query_builder: qb,
            inner: schema_manager,
        }
    }

    pub async fn fetch_all(self) -> Result<Option<Vec<StructuredColumnAndValue>>, anyhow::Error> {
        let results = self.inner.fetch_all(&self.query_builder).await;
        if let Ok(records) = results {
            match records {
                Some(rs) => Ok(Some(StructuredColumnAndValue::from_results(rs))),
                None => Ok(Some(Vec::new())),
            }
        } else {
            Err(results.err().unwrap())
        }
    }

    pub async fn all(self) -> Result<Option<Vec<StructuredColumnAndValue>>, anyhow::Error> {
        self.fetch_all().await
    }

    pub async fn get(self) -> Result<Option<Vec<StructuredColumnAndValue>>, anyhow::Error> {
        self.fetch_all().await
    }

    pub async fn get_to<T: FromColumnAndValue>(self) -> Result<Option<Vec<T>>, anyhow::Error> {
        self.fetch_all_to().await
    }

    pub async fn fetch_one(self) -> Result<Option<StructuredColumnAndValue>, anyhow::Error> {
        // self.query_builder.limit(1); // We shouldn't be doing this. It affects joins

        let result = self.inner.fetch_one(&self.query_builder).await;

        if let Ok(row) = result {
            match row {
                Some(r) => Ok(Some(StructuredColumnAndValue::from_a_result(r)?)),
                None => Ok(None),
            }
        } else {
            Err(result.err().unwrap())
        }
    }

    pub async fn first(self) -> Result<Option<StructuredColumnAndValue>, anyhow::Error> {
        self.fetch_one().await
    }

    pub async fn first_to<T: FromColumnAndValue>(self) -> Result<Option<T>, anyhow::Error> {
        self.fetch_one_to().await
    }

    pub async fn fetch_one_to<T: FromColumnAndValue>(self) -> Result<Option<T>, anyhow::Error>
    where
        Self: Sized,
    {
        let result = self.fetch_one().await;

        if let Ok(row) = result {
            match row {
                Some(r) => Ok(Some(T::from_column_value(r.fields())?)),
                None => Ok(None),
            }
        } else {
            Err(result.err().unwrap())
        }
    }

    pub async fn fetch_all_to<T>(self) -> Result<Option<Vec<T>>, anyhow::Error>
    where
        Self: Sized,
        T: FromColumnAndValue,
    {
        let result = self.fetch_all().await;
        if let Ok(records) = result {
            match records {
                Some(rows) => Ok(Some(
                    rows.into_iter()
                        .flat_map(|row| T::from_column_value(row.fields()))
                        .collect::<Vec<T>>(),
                )),
                None => Ok(Some(Vec::new())),
            }
        } else {
            Err(result.err().unwrap())
        }
    }

    pub async fn stream(self) -> tokio_stream::wrappers::ReceiverStream<ColumnAndValue> {
        let (sender, receiver) = tokio::sync::mpsc::channel::<ColumnAndValue>(100);

        tokio::spawn(async move {
            _ = self.inner.stream_result(&self.query_builder, sender).await;
        });

        tokio_stream::wrappers::ReceiverStream::new(receiver)
    }

    pub async fn stream_to<T: FromColumnAndValue + Send + Sync + 'static>(
        self,
    ) -> tokio_stream::wrappers::ReceiverStream<T> {
        let (inner_sender, mut inner_receiver) = tokio::sync::mpsc::channel::<ColumnAndValue>(100);
        let (outer_sender, outer_receiver) = tokio::sync::mpsc::channel::<T>(100);

        tokio::spawn(async move {
            while let Some(result) = inner_receiver.recv().await {
                let Ok(row) = StructuredColumnAndValue::from_a_result(result) else {
                    break;
                };
                match T::from_column_value(row.fields()) {
                    Ok(d) => {
                        if let Err(e) = outer_sender.send(d).await {
                            tracing::error!("error sending transformed row result: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("error sending transformed row result: {}", e);
                        break;
                    }
                }
            }
        });

        tokio::spawn(async move {
            _ = self
                .inner
                .stream_result(&self.query_builder, inner_sender)
                .await;
        });

        tokio_stream::wrappers::ReceiverStream::new(outer_receiver)
    }
}
