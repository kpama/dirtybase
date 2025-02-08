use crate::db::types::ArcUuid7;

use super::{TenantContext, TenantRepositoryTrait};

pub struct DummyTenantRepository;

#[async_trait::async_trait]
impl TenantRepositoryTrait for DummyTenantRepository {
    async fn by_id(&self, _id: ArcUuid7) -> Option<TenantContext> {
        tracing::trace!("dummy tenant manager `by_id`");
        None
    }
}
