use std::marker::PhantomData;

use crate::db_contract::{
    base::{manager::Manager, query::QueryBuilder},
    field_values::FieldValue,
    types::StructuredColumnAndValue,
    TableEntityTrait,
};

use super::{RelationMany, RelationOne, RelationQueryBuilder};

pub struct HasManyThrough<P, PV, C>
where
    P: TableEntityTrait,
    PV: TableEntityTrait,
    C: TableEntityTrait,
{
    pivot_field: String,
    pivot_child_field: String,
    pivot_table: String,
    child_field: String,
    child_table: String,
    pub(crate) query_builder: QueryBuilder,
    pub(crate) manager: Manager,
    parent_phantom: PhantomData<P>,
    pivot_phantom: PhantomData<PV>,
    child_phantom: PhantomData<C>,
}

impl<P, PV, C> HasManyThrough<P, PV, C>
where
    P: TableEntityTrait,
    PV: TableEntityTrait,
    C: TableEntityTrait,
{
    pub fn new(manager: Manager) -> Self {
        Self::new_with_custom(
            manager,
            PV::prefix_with_tbl(P::foreign_id_column().as_ref().unwrap()).as_str(),
            PV::prefix_with_tbl(C::foreign_id_column().as_ref().unwrap()).as_str(),
            PV::table_name(),
            C::prefix_with_tbl(C::id_column().as_ref().unwrap()).as_str(),
            C::table_name(),
        )
    }

    pub fn new_with_custom(
        manager: Manager,
        pivot_field: &str,
        pivot_child_field: &str,
        pivot_table: &str,
        child_field: &str,
        child_table: &str,
    ) -> Self {
        Self {
            manager,
            pivot_field: pivot_field.to_string(),
            pivot_child_field: pivot_child_field.to_string(),
            pivot_table: pivot_table.to_string(),
            child_field: child_field.to_string(),
            child_table: child_table.to_string(),
            query_builder: Self::make_query_builder(
                (),
                pivot_table,
                pivot_field,
                pivot_child_field,
                child_field,
                child_table,
            ),
            parent_phantom: PhantomData,
            pivot_phantom: PhantomData,
            child_phantom: PhantomData,
        }
    }

    pub async fn pivots(&mut self) -> Result<Option<Vec<PV>>, anyhow::Error> {
        self.manager
            .execute_query(self.query_builder.clone())
            .fetch_all_to()
            .await
    }

    pub async fn pivots_s(
        &mut self,
    ) -> Result<Option<Vec<StructuredColumnAndValue>>, anyhow::Error> {
        self.manager
            .execute_query(self.query_builder.clone())
            .fetch_all()
            .await
    }

    fn make_query_builder(
        value: impl Into<FieldValue>,
        pivot_table: &str,
        pivot_field: &str,
        pivot_child_field: &str,
        child_field: &str,
        child_table: &str,
    ) -> QueryBuilder {
        let mut qb = QueryBuilder::new(
            pivot_table,
            crate::db_contract::base::query::QueryAction::Query {
                columns: Some(PV::table_column_full_names()),
            },
        );

        qb.eq(pivot_field, value);

        qb.left_join_and_select(
            child_table,
            child_field,
            "=",
            pivot_child_field,
            &C::table_column_full_names(),
        );

        qb
    }
}

impl<P, PV, C> RelationQueryBuilder for HasManyThrough<P, PV, C>
where
    P: TableEntityTrait + Send,
    PV: TableEntityTrait + Send,
    C: TableEntityTrait + Send,
{
    type Target = C;

    fn constrain_keys<K: Into<crate::db_contract::field_values::FieldValue> + IntoIterator>(
        &mut self,
        keys: K,
    ) -> &mut Self {
        let value = if let FieldValue::Array(v) = keys.into() {
            v.into_iter().next().unwrap_or(FieldValue::Null)
        } else {
            FieldValue::Null
        };

        self.query_builder = Self::make_query_builder(
            value,
            &self.pivot_table,
            &self.pivot_field,
            &self.pivot_child_field,
            &self.child_field,
            &self.child_table,
        );

        self
    }

    fn query_builder(&mut self) -> &mut QueryBuilder {
        &mut self.query_builder
    }
}

#[async_trait::async_trait]
impl<P, PV, C> RelationOne for HasManyThrough<P, PV, C>
where
    P: TableEntityTrait + Send,
    PV: TableEntityTrait + Send,
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
impl<P, PV, C> RelationMany for HasManyThrough<P, PV, C>
where
    P: TableEntityTrait + Send,
    PV: TableEntityTrait + Send,
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
