use crate::{
    base::{manager::Manager, query::QueryBuilder},
    field_values::FieldValue,
    types::StructuredColumnAndValue,
    TableEntityTrait,
};

use super::{MorphManyToMany, RelationOne, RelationQueryBuilder};

pub struct MorphOneToOne<P, C>
where
    P: TableEntityTrait,
    C: TableEntityTrait,
{
    relation: MorphManyToMany<P, C>,
}

impl<P, C> MorphOneToOne<P, C>
where
    P: TableEntityTrait,
    C: TableEntityTrait,
{
    pub fn new(manager: Manager, child_field: &str, child_type_field: &str) -> Self {
        Self::new_with_custom(
            manager,
            child_field,
            P::table_name(),
            child_type_field,
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
        Self {
            relation: MorphManyToMany::new_with_custom(
                manager,
                child_field,
                child_type,
                child_type_field,
                child_table,
            ),
        }
    }
}

impl<P, C> RelationQueryBuilder for MorphOneToOne<P, C>
where
    P: TableEntityTrait + Send,
    C: TableEntityTrait + Send,
{
    type Target = C;

    fn constrain_keys<K: Into<FieldValue> + IntoIterator>(&mut self, keys: K) -> &mut Self {
        let mut qb = QueryBuilder::new(
            C::table_name(),
            crate::base::query::QueryAction::Query {
                columns: Some(C::table_column_full_names()),
            },
        );

        let value = if let FieldValue::Array(v) = keys.into() {
            v.into_iter().next().unwrap_or(FieldValue::Null)
        } else {
            FieldValue::Null
        };
        qb.eq(&self.relation.child_field, value);
        qb.eq(&self.relation.child_type_field, &self.relation.child_type);

        self.relation.query_builder = qb;
        self
    }

    fn query_builder(&mut self) -> &mut QueryBuilder {
        self.relation.query_builder()
    }
}

#[async_trait::async_trait]
impl<P, C> RelationOne for MorphOneToOne<P, C>
where
    P: TableEntityTrait + Send,
    C: TableEntityTrait + Send,
{
    async fn one_s(&mut self) -> Result<Option<StructuredColumnAndValue>, anyhow::Error> {
        self.relation.one_s().await
    }

    async fn one(&mut self) -> Result<Option<Self::Target>, anyhow::Error> {
        self.relation.one().await
    }
}
