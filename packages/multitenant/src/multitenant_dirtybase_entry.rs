use dirtybase_contract::{
    async_trait, busybody,
    config::DirtyConfig,
    multitenant::{TenantRepositoryProvider, TenantResolverProvider},
};

use crate::MultitenantConfig;

#[derive(Debug, Default)]
pub struct Extension;

#[async_trait]
impl dirtybase_contract::ExtensionSetup for Extension {
    async fn setup(&mut self, base: &DirtyConfig) {
        let config = MultitenantConfig::from(base);
        tracing::error!("id location: {:?}", config.id_location());

        // Default repository aka Dummy repository
        busybody::helpers::service_container().set(TenantRepositoryProvider::default());
        // Default resolver
        busybody::helpers::service_container().set(TenantResolverProvider::default());
    }
}
