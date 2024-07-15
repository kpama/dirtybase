use std::sync::Arc;

use crate::{
    base::{manager::Manager, query::QueryBuilder},
    field_values::FieldValue,
    types::StructuredColumnAndValue,
    TableEntityTrait,
};

use super::{MorphManyToMany, RelationQueryBuilder};

pub struct MorphOneOfMany<P: TableEntityTrait, C: TableEntityTrait> {
    relation: MorphManyToMany<P, C>,
}

impl<P, C> MorphOneOfMany<P, C>
where
    P: TableEntityTrait,
    C: TableEntityTrait,
{
    pub fn new(manager: Arc<Manager>, child_field: &str, child_type_field: &str) -> Self {
        Self::new_with_custom(
            manager,
            child_field,
            P::table_name(),
            child_type_field,
            C::table_name(),
        )
    }

    pub fn new_with_custom(
        manager: Arc<Manager>,
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

    pub async fn latest_of_many(
        &mut self,
        sort_by: Option<&str>,
    ) -> Result<Option<C>, anyhow::Error>
    where
        P: TableEntityTrait + Send,
        C: TableEntityTrait + Send,
    {
        let qb = self.buld_query(sort_by, false);
        self.relation.manager.execute_query(qb).fetch_one_to().await
    }

    pub async fn latest_of_many_s(
        &mut self,
        sort_by: Option<&str>,
    ) -> Result<Option<StructuredColumnAndValue>, anyhow::Error>
    where
        P: TableEntityTrait + Send,
        C: TableEntityTrait + Send,
    {
        let qb = self.buld_query(sort_by, false);
        self.relation.manager.execute_query(qb).fetch_one().await
    }

    pub async fn oldest_of_many(
        &mut self,
        sort_by: Option<&str>,
    ) -> Result<Option<C>, anyhow::Error>
    where
        P: TableEntityTrait + Send,
        C: TableEntityTrait + Send,
    {
        let qb = self.buld_query(sort_by, true);
        self.relation.manager.execute_query(qb).fetch_one_to().await
    }

    pub async fn oldest_of_many_s(
        &mut self,
        sort_by: Option<&str>,
    ) -> Result<Option<StructuredColumnAndValue>, anyhow::Error>
    where
        P: TableEntityTrait + Send,
        C: TableEntityTrait + Send,
    {
        let qb = self.buld_query(sort_by, true);
        self.relation.manager.execute_query(qb).fetch_one().await
    }

    fn buld_query(&self, sort_by: Option<&str>, asc: bool) -> QueryBuilder {
        let mut qb = self.relation.query_builder.clone();
        let sort_by = if let Some(f) = sort_by {
            f.to_string()
        } else {
            C::id_column().unwrap().to_string()
        };

        if asc {
            qb.asc(sort_by);
        } else {
            qb.desc(sort_by);
        }
        qb
    }
}

impl<P, C> RelationQueryBuilder for MorphOneOfMany<P, C>
where
    P: TableEntityTrait + Send,
    C: TableEntityTrait + Send,
{
    type Target = C;

    fn constrain_keys<K: Into<FieldValue> + IntoIterator>(&mut self, keys: K) {
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
    }
}
