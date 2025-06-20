use std::marker::PhantomData;

use crate::db_contract::{
    base::{manager::Manager, query::QueryBuilder},
    field_values::FieldValue,
    types::StructuredColumnAndValue,
    TableEntityTrait,
};

use super::{RelationMany, RelationOne, RelationQueryBuilder};

pub struct HasMany<P, C>
where
    P: TableEntityTrait,
    C: TableEntityTrait,
{
    child_field: String,
    child_table: String,
    manager: Manager,
    query_builder: QueryBuilder,
    parent_phantom: PhantomData<P>,
    child_phantom: PhantomData<C>,
}

impl<P, C> HasMany<P, C>
where
    P: TableEntityTrait,
    C: TableEntityTrait,
{
    pub fn new(manager: Manager) -> Self {
        Self::new_with_custom(
            manager,
            C::prefix_with_tbl(P::foreign_id_column()).as_str(),
            C::table_name(),
        )
    }

    pub fn new_with_custom(manager: Manager, child_field: &str, child_table: &str) -> Self {
        let mut qb = QueryBuilder::new(
            child_table,
            crate::db_contract::base::query::QueryAction::Query {
                columns: Some(C::table_query_columns()),
            },
        );

        qb.is_in(child_field, Vec::<&str>::new());

        Self {
            child_field: child_field.to_string(),
            child_table: child_table.to_string(),
            query_builder: qb,
            parent_phantom: PhantomData,
            child_phantom: PhantomData,
            manager,
        }
    }

    pub async fn count(&self) -> i64 {
        let mut qb = self.query_builder.clone();
        qb.count_as(&self.child_field, "has_many_total");
        if let Ok(Some(r)) = self.manager.execute_query(qb).first().await {
            r.get("has_many_total").unwrap().into()
        } else {
            0
        }
    }
}

impl<P, C> RelationQueryBuilder for HasMany<P, C>
where
    P: TableEntityTrait + Send,
    C: TableEntityTrait + Send,
{
    type Target = C;

    fn constrain_keys<K: Into<FieldValue> + IntoIterator>(&mut self, keys: K) -> &mut Self {
        let mut qb = QueryBuilder::new(
            &self.child_table,
            crate::db_contract::base::query::QueryAction::Query {
                columns: Some(C::table_query_columns()),
            },
        );
        qb.is_in(&self.child_field, keys);

        self.query_builder = qb;

        self
    }

    fn query_builder(&mut self) -> &mut QueryBuilder {
        &mut self.query_builder
    }
}

#[async_trait::async_trait]
impl<P, C> RelationMany for HasMany<P, C>
where
    P: TableEntityTrait + Send,
    C: TableEntityTrait + Send,
{
    async fn get(&mut self) -> Result<Option<Vec<Self::Target>>, anyhow::Error> {
        self.manager
            .execute_query(self.query_builder.clone())
            .fetch_all_to()
            .await
    }

    async fn get_s(&mut self) -> Result<Option<Vec<StructuredColumnAndValue>>, anyhow::Error> {
        self.manager
            .execute_query(self.query_builder.clone())
            .fetch_all()
            .await
    }
}

#[async_trait::async_trait]
impl<P, C> RelationOne for HasMany<P, C>
where
    P: TableEntityTrait + Send,
    C: TableEntityTrait + Send,
{
    async fn one_s(&mut self) -> Result<Option<StructuredColumnAndValue>, anyhow::Error> {
        self.manager
            .execute_query(self.query_builder.clone())
            .fetch_one()
            .await
    }

    async fn one(&mut self) -> Result<Option<Self::Target>, anyhow::Error> {
        self.manager
            .execute_query(self.query_builder.clone())
            .fetch_one_to()
            .await
    }
}
