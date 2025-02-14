use std::marker::PhantomData;

use crate::db::{
    base::{manager::Manager, query::QueryBuilder},
    field_values::FieldValue,
    types::StructuredColumnAndValue,
    TableEntityTrait,
};

use super::{RelationMany, RelationOne, RelationQueryBuilder};

pub struct MorphManyToMany<P, C>
where
    P: TableEntityTrait,
    C: TableEntityTrait,
{
    pub(crate) child_field: String,
    pub(crate) child_type_field: String,
    pub(crate) child_type: String,
    pub(crate) child_table: String,
    pub(crate) query_builder: QueryBuilder,
    parent_phantom: PhantomData<P>,
    child_phantom: PhantomData<C>,
    pub(crate) manager: Manager,
}

impl<P, C> MorphManyToMany<P, C>
where
    P: TableEntityTrait,
    C: TableEntityTrait,
{
    pub fn new(manager: Manager, child_field: &str, child_type_field: &str) -> Self {
        Self::new_with_custom(
            manager,
            C::prefix_with_tbl(child_field).as_str(),
            P::table_name(),
            C::prefix_with_tbl(child_type_field).as_str(),
            C::table_name(),
        )
    }

    pub fn new_with_custom(
        manager: Manager,
        child_field: &str,
        child_type: &str,
        child_type_field: &str,
        child_table: &str,
    ) -> Self {
        let mut qb = QueryBuilder::new(
            child_table,
            crate::db::base::query::QueryAction::Query {
                columns: Some(C::table_column_full_names()),
            },
        );

        qb.is_in(child_field, Vec::<&str>::new());
        qb.eq(child_type_field, child_type);

        Self {
            manager,
            query_builder: qb,
            child_field: child_field.to_string(),
            child_type: child_type.to_string(),
            child_type_field: child_type_field.to_string(),
            child_table: child_table.to_string(),
            child_phantom: PhantomData,
            parent_phantom: PhantomData,
        }
    }
}

impl<P, C> RelationQueryBuilder for MorphManyToMany<P, C>
where
    P: TableEntityTrait + Send,
    C: TableEntityTrait + Send,
{
    type Target = C;

    fn constrain_keys<K: Into<FieldValue> + IntoIterator>(&mut self, keys: K) -> &mut Self {
        let mut qb = QueryBuilder::new(
            &self.child_table,
            crate::db::base::query::QueryAction::Query {
                columns: Some(C::table_column_full_names()),
            },
        );

        qb.is_in(&self.child_field, keys);
        qb.eq(&self.child_type_field, &self.child_type);

        self.query_builder = qb;

        self
    }

    fn query_builder(&mut self) -> &mut QueryBuilder {
        &mut self.query_builder
    }
}

#[async_trait::async_trait]
impl<P, C> RelationOne for MorphManyToMany<P, C>
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

#[async_trait::async_trait]
impl<P, C> RelationMany for MorphManyToMany<P, C>
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
