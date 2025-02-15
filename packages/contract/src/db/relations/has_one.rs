use crate::db::{
    base::manager::Manager, field_values::FieldValue, types::StructuredColumnAndValue,
    TableEntityTrait,
};

use super::{HasMany, RelationOne, RelationQueryBuilder};

pub struct HasOne<P, C>
where
    P: TableEntityTrait,
    C: TableEntityTrait,
{
    relation: HasMany<P, C>,
}

impl<P, C> HasOne<P, C>
where
    P: TableEntityTrait,
    C: TableEntityTrait,
{
    pub fn new(manager: Manager) -> Self {
        Self::new_with_custom(
            manager,
            C::prefix_with_tbl(P::foreign_id_column().as_ref().unwrap()).as_str(),
            C::table_name(),
        )
    }

    pub fn new_with_custom(manager: Manager, child_field: &str, child_table: &str) -> Self {
        Self {
            relation: HasMany::new_with_custom(manager, child_field, child_table),
        }
    }
}

impl<P, C> RelationQueryBuilder for HasOne<P, C>
where
    P: TableEntityTrait + Send,
    C: TableEntityTrait + Send,
{
    type Target = C;

    fn constrain_keys<K: Into<FieldValue> + IntoIterator>(&mut self, keys: K) -> &mut Self {
        self.relation.constrain_keys(keys);
        self
    }

    fn query_builder(&mut self) -> &mut crate::db::base::query::QueryBuilder {
        self.relation.query_builder()
    }
}

#[async_trait::async_trait]
impl<P, C> RelationOne for HasOne<P, C>
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
