mod event;
mod event_handler;
mod http;
mod migration;
mod model;

use dirtybase_contract::prelude::*;

use crate::MultitenantConfig;

#[derive(Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl dirtybase_contract::ExtensionSetup for Extension {
    async fn setup(&mut self, base: &DirtyConfig) {
        event_handler::setup().await;
        let config = MultitenantConfig::from(base);
    }

    fn migrations(&self) -> Option<dirtybase_contract::ExtensionMigrations> {
        migration::setup()
    }

    fn register_routes(
        &self,
        mut manager: RouterManager,
        _middleware: &WebMiddlewareManager,
    ) -> RouterManager {
        manager
    }
}
