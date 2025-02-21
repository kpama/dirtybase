mod event;
mod event_handler;
mod http;
mod migration;
mod model;

use dirtybase_contract::prelude::*;

#[derive(Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl dirtybase_contract::ExtensionSetup for Extension {
    async fn setup(&mut self, _config: &DirtyConfig) {
        event_handler::setup().await;
    }

    fn migrations(&self) -> Option<dirtybase_contract::ExtensionMigrations> {
        migration::setup()
    }
}
