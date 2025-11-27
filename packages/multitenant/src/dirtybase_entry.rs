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
        _ = context
            .load_config::<MultitenantConfig>("multitenant")
            .await
            .expect("could not load multi tenant configuration");
    }

    fn migrations(&self, _context: &Context) -> Option<dirtybase_contract::ExtensionMigrations> {
        migration::setup()
    }
}
