mod event;
mod event_handler;
mod http;
mod migration;
mod model;

use dirtybase_contract::dirtybase_config::DirtyConfig;

pub struct Extension;

#[dirtybase_contract::async_trait]
impl dirtybase_contract::ExtensionSetup for Extension {
    async fn setup(&self, _config: &DirtyConfig) {
        event_handler::setup().await;
    }

    fn migrations(&self) -> dirtybase_contract::ExtensionMigrations {
        migration::setup()
    }
}
