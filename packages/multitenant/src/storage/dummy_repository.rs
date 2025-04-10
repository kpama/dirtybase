use dirtybase_contract::{
    async_trait,
    db_contract::types::ArcUuid7,
    multitenant_contract::{TenantContext, TenantStorageTrait},
};

pub struct DummyTenantRepository;

#[async_trait]
impl TenantStorageTrait for DummyTenantRepository {
    async fn by_id(&self, _id: ArcUuid7) -> Option<TenantContext> {
        tracing::trace!("dummy tenant manager `by_id`");
        None
    }
}
