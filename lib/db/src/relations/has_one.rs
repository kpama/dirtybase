use std::{marker::PhantomData, sync::Arc};

use crate::{
    base::{manager::Manager, query::QueryBuilder},
    field_values::FieldValue,
    types::StructuredColumnAndValue,
    TableEntityTrait,
};

use super::{RelationOne, RelationQueryBuilder};

pub struct HasOne<P, C>
where
    P: TableEntityTrait,
    C: TableEntityTrait,
{
    child_field: String,
    child_table: String,
    query_builder: QueryBuilder,
    parent_phantom: PhantomData<P>,
    child_phantom: PhantomData<C>,
    manager: Arc<Manager>,
}

impl<P, C> HasOne<P, C>
where
    P: TableEntityTrait,
    C: TableEntityTrait,
{
    pub fn new(manager: Arc<Manager>) -> Self {
        Self::new_with_custom(
            manager,
            P::foreign_id_column().as_ref().unwrap(),
            C::table_name(),
        )
    }

    pub fn new_with_custom(manager: Arc<Manager>, child_field: &str, child_table: &str) -> Self {
        let mut qb = QueryBuilder::new(
            child_table,
            crate::base::query::QueryAction::Query {
                columns: Some(C::table_column_full_names()),
            },
        );

        qb.eq(child_field, FieldValue::Null);
        Self {
            child_field: child_field.to_string(),
            child_table: child_table.to_string(),
            query_builder: qb,
            parent_phantom: PhantomData,
            child_phantom: PhantomData,
            manager,
        }
    }
}

impl<P, C> RelationQueryBuilder for HasOne<P, C>
where
    P: TableEntityTrait + Send,
    C: TableEntityTrait + Send,
{
    type Target = C;

    fn constrain_keys<K: Into<FieldValue> + IntoIterator>(&mut self, keys: K) {
        let mut qb = QueryBuilder::new(
            &self.child_table,
            crate::base::query::QueryAction::Query {
                columns: Some(C::table_column_full_names()),
            },
        );

        let value = if let FieldValue::Array(v) = keys.into() {
            v.into_iter().next().unwrap_or(FieldValue::Null)
        } else {
            FieldValue::Null
        };
        qb.eq(&self.child_field, value);

        self.query_builder = qb;
    }
}

#[async_trait::async_trait]
impl<P, C> RelationOne for HasOne<P, C>
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
