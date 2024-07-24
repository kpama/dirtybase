use crate::{base::manager::Manager, types::StructuredColumnAndValue, TableEntityTrait};

use super::{HasManyThrough, RelationOne, RelationQueryBuilder};

pub struct HasOneThrough<P, PV, C>
where
    P: TableEntityTrait,
    PV: TableEntityTrait,
    C: TableEntityTrait,
{
    relation: HasManyThrough<P, PV, C>,
}

impl<P, PV, C> HasOneThrough<P, PV, C>
where
    P: TableEntityTrait,
    PV: TableEntityTrait,
    C: TableEntityTrait,
{
    pub fn new(manager: Manager) -> Self {
        Self::new_with_custom(
            manager,
            P::foreign_id_column().as_ref().unwrap(),
            C::foreign_id_column().as_ref().unwrap(),
            PV::table_name(),
            C::id_column().as_ref().unwrap(),
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
            relation: HasManyThrough::new_with_custom(
                manager,
                pivot_field,
                pivot_child_field,
                pivot_table,
                child_field,
                child_table,
            ),
        }
    }

    pub async fn pivot(&mut self) -> Result<Option<PV>, anyhow::Error> {
        self.relation
            .manager
            .execute_query(self.relation.query_builder.clone())
            .fetch_one_to()
            .await
    }
}

impl<P, PV, C> RelationQueryBuilder for HasOneThrough<P, PV, C>
where
    P: TableEntityTrait + Send,
    PV: TableEntityTrait + Send,
    C: TableEntityTrait + Send,
{
    type Target = C;

    fn constrain_keys<K: Into<crate::field_values::FieldValue> + IntoIterator>(
        &mut self,
        keys: K,
    ) -> &mut Self {
        self.relation.constrain_keys(keys);

        self
    }

    fn query_builder(&mut self) -> &mut crate::base::query::QueryBuilder {
        self.relation.query_builder()
    }
}

#[async_trait::async_trait]
impl<P, PV, C> RelationOne for HasOneThrough<P, PV, C>
where
    P: TableEntityTrait + Send,
    PV: TableEntityTrait + Send,
    C: TableEntityTrait + Send,
{
    async fn one_s(&mut self) -> Result<Option<StructuredColumnAndValue>, anyhow::Error> {
        self.relation.one_s().await
    }

    async fn one(&mut self) -> Result<Option<Self::Target>, anyhow::Error> {
        self.relation.one().await
    }
}
