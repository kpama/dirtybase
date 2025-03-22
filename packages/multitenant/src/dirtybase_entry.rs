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
    async fn setup(&mut self, context: &Context) {
        event_handler::setup().await;
        let config = context
            .get_config::<MultitenantConfig>("dirtybase::multitenant")
            .await
            .unwrap();
    }

    fn migrations(&self, _context: &Context) -> Option<dirtybase_contract::ExtensionMigrations> {
        migration::setup()
    }

    fn register_routes(&self, manager: RouterManager) -> RouterManager {
        manager
    }
}
