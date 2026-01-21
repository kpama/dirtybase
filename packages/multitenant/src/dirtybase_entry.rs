mod http;
mod migration;
mod model;
mod resource_manager;

pub mod storage;

use dirtybase_contract::{
    http_contract::prelude::*,
    multitenant_contract::{RequestTenantResolver, TenantManager},
    prelude::*,
};

use crate::{
    MultitenantConfig, dirtybase_entry::resource_manager::register_multitenant_resource_manager,
};

#[derive(Default)]
pub struct Extension {
    config: MultitenantConfig,
}

#[dirtybase_contract::async_trait]
impl dirtybase_contract::ExtensionSetup for Extension {
    async fn setup(&mut self, context: &Context) {
        register_multitenant_resource_manager().await;

        self.config = context
            .get_config_once::<MultitenantConfig>("multitenant")
            .await
            .expect("could not load multi tenant configuration");

        context
            .container()
            .resolver(|_| async { RequestTenantResolver::new() })
            .await;
    }

    async fn on_web_request(
        &self,
        req: Request,
        _context: Context,
        _cookie: &CookieJar,
    ) -> Request {
        req
    }

    fn migrations(&self, _context: &Context) -> Option<dirtybase_contract::ExtensionMigrations> {
        migration::setup()
    }
}
