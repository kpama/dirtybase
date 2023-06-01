use async_trait::async_trait;
use sqlx::{any::AnyKind, MySql, Pool};
use std::{collections::HashMap, sync::Arc};

use crate::driver::surreal::SurrealClient;

use super::{field_values::FieldValue, query::QueryBuilder, table::BaseTable};

pub trait RelationalDbTrait: SchemaManagerTrait {
    fn instance(db_pool: Arc<Pool<MySql>>) -> Self
    where
        Self: Sized;
    fn kind(&self) -> AnyKind;
}

pub trait SurrealDbTrait: SchemaManagerTrait {
    fn instance(client: Arc<SurrealClient>) -> Self
    where
        Self: Sized;
    fn inner_client(&self) -> Arc<SurrealClient>;
}

pub trait GraphDbClient<T = SurrealClient>: Send {
    fn into_inner_client(&self) -> Arc<T>;
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

    async fn save(&self, query_builder: QueryBuilder);

    async fn fetch_all_as_json(&self) -> Vec<serde_json::Value>;

    async fn fetch_all_as_field_value(&self) -> Vec<HashMap<String, FieldValue>>;

    // checks if a table exist in the database
    async fn has_table(&self, name: &str) -> bool;
}
