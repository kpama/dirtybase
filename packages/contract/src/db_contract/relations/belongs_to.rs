use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::db_contract::{
    base::{manager::Manager, query::QueryBuilder},
    field_values::FieldValue,
    types::StructuredColumnAndValue,
    TableEntityTrait,
};

use super::{RelationOne, RelationQueryBuilder};

pub struct BelongsTo<C, P>
where
    C: TableEntityTrait,
    P: TableEntityTrait,
{
    parent_field: String,
    parent_table: String,
    query_builder: QueryBuilder,
    manager: Manager,
    child_phantom: PhantomData<C>,
    parent_phantom: PhantomData<P>,
}

impl<C, P> BelongsTo<C, P>
where
    C: TableEntityTrait,
    P: TableEntityTrait,
{
    pub fn new(manager: Manager) -> Self {
        Self::new_with_custom(
            manager,
            P::prefix_with_tbl(P::id_column()).as_str(),
            P::table_name(),
        )
    }

    pub fn new_with_custom(manager: Manager, parent_field: &str, parent_table: &str) -> Self {
        let mut qb = QueryBuilder::new(
            parent_table,
            crate::db_contract::base::query::QueryAction::Query {
                columns: Some(P::table_query_columns()),
            },
        );

        qb.is_in(parent_field, Vec::<String>::new());

        Self {
            parent_field: parent_field.to_string(),
            parent_table: parent_table.to_string(),
            query_builder: qb,
            child_phantom: PhantomData,
            parent_phantom: PhantomData,
            manager,
        }
    }
}

impl<C, P> RelationQueryBuilder for BelongsTo<C, P>
where
    C: TableEntityTrait + Send,
    P: TableEntityTrait + Send,
{
    type Target = P;

    fn constrain_keys<K: Into<FieldValue> + IntoIterator>(&mut self, keys: K) -> &mut Self {
        let mut qb = QueryBuilder::new(
            &self.parent_table,
            crate::db_contract::base::query::QueryAction::Query {
                columns: Some(P::table_query_columns()),
            },
        );
        qb.is_in(&self.parent_field, keys);

        self.query_builder = qb;

        self
    }

    fn query_builder(&mut self) -> &mut QueryBuilder {
        &mut self.query_builder
    }
}

#[async_trait::async_trait]
impl<C, P> RelationOne for BelongsTo<C, P>
where
    C: TableEntityTrait + Send,
    P: TableEntityTrait + Send,
{
    async fn one(&mut self) -> Result<Option<Self::Target>, anyhow::Error> {
        self.manager
            .execute_query(self.query_builder.clone())
            .fetch_one_to()
            .await
    }

    async fn one_s(&mut self) -> Result<Option<StructuredColumnAndValue>, anyhow::Error> {
        self.manager
            .execute_query(self.query_builder.clone())
            .fetch_one()
            .await
    }
}

impl<C, P> Deref for BelongsTo<C, P>
where
    C: TableEntityTrait,
    P: TableEntityTrait,
{
    type Target = QueryBuilder;
    fn deref(&self) -> &Self::Target {
        &self.query_builder
    }
}

impl<C, P> DerefMut for BelongsTo<C, P>
where
    C: TableEntityTrait,
    P: TableEntityTrait,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.query_builder
    }
}
