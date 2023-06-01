#![allow(dead_code)]

use super::SurrealClient;
use crate::base::{
    field_values::FieldValue,
    query::QueryBuilder,
    schema::{GraphDbClient, GraphDbTrait, SchemaManagerTrait, SurrealDbTrait},
    table::BaseTable,
};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};

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

    fn build_query(&self, _query: &QueryBuilder, _params: &mut Vec<String>) -> String {
        let sql = "SELECT * FROM type:table(family)".to_owned();

        sql
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

    fn inner_client(&self) -> Arc<SurrealClient> {
        self.client.clone()
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
    fn fetch_table_for_update(&self, _name: &str) -> BaseTable {
        todo!()
    }

    async fn commit(&self, _table: BaseTable) {
        todo!()
    }
    fn query(&mut self, query_builder: QueryBuilder) -> &dyn SchemaManagerTrait {
        let mut params = Vec::new();
        let statement = self.build_query(&query_builder, &mut params);

        self.active_query = Some(ActiveQuery { statement, params });

        self
    }

    async fn save(&self, _query_builder: QueryBuilder) {
        todo!()
    }

    async fn fetch_all_as_json(&self) -> Vec<serde_json::Value> {
        let mut results: Vec<serde_json::Value> = Vec::new();
        match &self.active_query {
            Some(active_query) => {
                let client = self.client.query(&active_query.statement);
                match client.await {
                    Ok(mut response) => {
                        results = response.take(0).unwrap_or_default();
                    }
                    Err(e) => {
                        log::error!("could not fetch data: {}", e.to_string());
                    }
                }
            }
            None => (),
        }

        results
    }

    async fn fetch_all_as_field_value(&self) -> Vec<HashMap<String, FieldValue>> {
        Vec::new()
    }

    async fn has_table(&self, name: &str) -> bool {
        let query = "INFO FOR DB";

        match self.client.query(query).await {
            Ok(mut response) => {
                let table_key = "tb";
                let result_index = 0usize;
                let tables: Option<serde_json::Value> =
                    response.take((result_index, table_key)).unwrap_or(None);

                if tables.is_some() && tables.unwrap().get(name).is_some() {
                    return true;
                }
                false
            }
            Err(_) => false,
        }
    }
}
