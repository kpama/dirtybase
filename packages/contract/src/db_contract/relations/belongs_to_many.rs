use std::marker::PhantomData;

use crate::db_contract::{
    base::{manager::Manager, query::QueryBuilder},
    field_values::FieldValue,
    types::StructuredColumnAndValue,
    TableEntityTrait,
};

use super::{RelationMany, RelationOne, RelationQueryBuilder};

pub struct BelongsToMany<C, PV, P>
where
    C: TableEntityTrait,
    PV: TableEntityTrait,
    P: TableEntityTrait,
{
    pivot_field: String,
    pivot_parent_field: String,
    pivot_table: String,
    parent_field: String,
    parent_table: String,
    child_phantom: PhantomData<C>,
    pivot_phantom: PhantomData<PV>,
    parent_phantom: PhantomData<P>,
    query_builder: QueryBuilder,
    manager: Manager,
}

impl<C, PV, P> BelongsToMany<C, PV, P>
where
    C: TableEntityTrait,
    PV: TableEntityTrait,
    P: TableEntityTrait,
{
    pub fn new(manager: Manager) -> Self {
        Self::new_with_custom(
            manager,
            PV::prefix_with_tbl(C::foreign_id_column().as_ref().unwrap()).as_str(),
            PV::prefix_with_tbl(P::foreign_id_column().as_ref().unwrap()).as_str(),
            PV::table_name(),
            P::prefix_with_tbl(P::id_column().as_ref().unwrap()).as_str(),
            P::table_name(),
            Vec::<String>::new(),
        )
    }

    pub fn new_with_custom<V: Into<FieldValue> + IntoIterator>(
        manager: Manager,
        pivot_field: &str,
        pivot_parent_field: &str,
        pivot_table: &str,
        parent_field: &str,
        parent_table: &str,
        values: V,
    ) -> Self {
        let mut qb = QueryBuilder::new(
            pivot_table,
            crate::db_contract::base::query::QueryAction::Query {
                columns: Some(PV::table_query_columns()),
            },
        );

        qb.is_in(pivot_field, values);

        qb.inner_join_and_select(
            parent_table,
            parent_field,
            "=",
            pivot_parent_field,
            &P::table_column_full_names(),
        );

        Self {
            pivot_field: pivot_field.to_string(),
            pivot_table: pivot_table.to_string(),
            pivot_parent_field: pivot_parent_field.to_string(),
            parent_field: parent_field.to_string(),
            parent_table: parent_table.to_string(),
            child_phantom: PhantomData,
            pivot_phantom: PhantomData,
            parent_phantom: PhantomData,
            query_builder: qb,
            manager,
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
}

impl<C, PV, P> RelationQueryBuilder for BelongsToMany<C, PV, P>
where
    C: TableEntityTrait,
    PV: TableEntityTrait,
    P: TableEntityTrait,
{
    type Target = P;

    fn constrain_keys<K: Into<FieldValue> + IntoIterator>(&mut self, keys: K) -> &mut Self {
        *self = Self::new_with_custom(
            self.manager.clone(),
            &self.pivot_field,
            &self.pivot_parent_field,
            &self.pivot_table,
            &self.parent_field,
            &self.parent_table,
            keys,
        );

        self
    }

    fn query_builder(&mut self) -> &mut QueryBuilder {
        &mut self.query_builder
    }
}

#[async_trait::async_trait]
impl<C, PV, P> RelationOne for BelongsToMany<C, PV, P>
where
    C: TableEntityTrait + Send,
    PV: TableEntityTrait + Send,
    P: TableEntityTrait + Send,
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
impl<C, PV, P> RelationMany for BelongsToMany<C, PV, P>
where
    C: TableEntityTrait + Send,
    PV: TableEntityTrait + Send,
    P: TableEntityTrait + Send,
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
