mod belongs_to;
mod belongs_to_many;
mod has_many;
mod has_many_through;
mod has_one;
mod has_one_through;
mod morph_many_to_many;
mod morph_one_of_many;
mod morph_one_to_many;
mod morph_one_to_one;

use super::{
    base::query::QueryBuilder, field_values::FieldValue, types::StructuredColumnAndValue,
    TableModel,
};

pub use belongs_to::*;
pub use belongs_to_many::*;
pub use has_many::*;
pub use has_many_through::*;
pub use has_one::*;
pub use has_one_through::*;
pub use morph_many_to_many::*;
pub use morph_one_of_many::*;
pub use morph_one_to_many::*;
pub use morph_one_to_one::*;

pub trait RelationQueryBuilder {
    type Target: TableModel;

    fn constrain_keys<K: Into<FieldValue> + IntoIterator>(&mut self, keys: K) -> &mut Self;

    fn constrain_key<K: Into<FieldValue>>(&mut self, key: K) {
        self.constrain_keys(vec![key]);
    }

    fn owner_key<K: Into<FieldValue>>(&mut self, key: K) {
        self.constrain_keys(vec![key]);
    }

    fn parent_key<K: Into<FieldValue>>(&mut self, key: K) {
        self.constrain_keys(vec![key]);
    }

    fn query_builder(&mut self) -> &mut QueryBuilder;
}

#[async_trait::async_trait]
pub trait RelationOne: RelationQueryBuilder {
    async fn one(&mut self) -> Result<Option<Self::Target>, anyhow::Error>;

    async fn one_s(&mut self) -> Result<Option<StructuredColumnAndValue>, anyhow::Error>;
}

#[async_trait::async_trait]
pub trait RelationMany: RelationQueryBuilder {
    async fn get(&mut self) -> Result<Option<Vec<Self::Target>>, anyhow::Error>;

    async fn get_s(&mut self) -> Result<Option<Vec<StructuredColumnAndValue>>, anyhow::Error>;
}

pub trait QueryBuildable: RelationQueryBuilder {
    fn query(&mut self) -> &mut QueryBuilder;
}
