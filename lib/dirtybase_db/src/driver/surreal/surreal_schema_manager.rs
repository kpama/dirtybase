use crate::base::{
    query::QueryBuilder,
    schema::{GraphDbClient, GraphDbTrait, SchemaManagerTrait, SurrealDbTrait},
    table::BaseTable,
};
use async_trait::async_trait;
use std::sync::Arc;

use super::SurrealClient;

struct ActiveQuery {
    statement: String,
    params: Vec<String>,
}

impl ActiveQuery {
    fn to_sql_string(&self) -> String {
        let mut query = self.statement.clone();
        for a_param in &self.params {
            query = query.replacen('?', a_param, 1);
        }

        query
    }
}
pub struct SurrealSchemaManager {
    client: Arc<SurrealClient>,
    active_query: Option<ActiveQuery>,
}

pub struct SurrealGraphDbClient {
    pub client: Arc<SurrealClient>,
}

impl SurrealGraphDbClient {
    pub fn new(client: Arc<SurrealClient>) -> Self {
        Self { client }
    }
}

impl SurrealSchemaManager {
    pub fn new(client: Arc<SurrealClient>) -> Self {
        Self {
            client,
            active_query: None,
        }
    }
}

#[async_trait]
impl SurrealDbTrait for SurrealSchemaManager {
    fn instance(client: Arc<SurrealClient>) -> Self
    where
        Self: Sized,
    {
        Self::new(client)
    }
}

impl GraphDbClient<SurrealClient> for SurrealGraphDbClient {
    fn into_inner_client(&self) -> Arc<SurrealClient> {
        self.client.clone()
    }
}

impl GraphDbTrait<SurrealClient> for SurrealSchemaManager {
    fn instance(client: Arc<SurrealClient>) -> Self
    where
        Self: Sized,
    {
        Self::new(client)
    }
}

#[async_trait]
impl SchemaManagerTrait for SurrealSchemaManager {
    fn fetch_table_for_update(&self, name: &str) -> BaseTable {
        todo!()
    }

    async fn commit(&self, table: BaseTable) {
        todo!()
    }
    fn query(&mut self, query_builder: QueryBuilder) -> &dyn SchemaManagerTrait {
        todo!()
    }

    async fn save(&self, query_builder: QueryBuilder) {
        todo!()
    }

    async fn fetch_all_as_json(&self) -> Vec<serde_json::Value> {
        todo!()
    }

    async fn has_table(&self, name: &str) -> bool {
        log::error!("making a request to surrealdb");

        match self.client.query(format!("info for table {}", name)).await {
            Ok(result) => {
                dbg!(result);
                true
            }
            Err(_) => false,
        }
    }
}
