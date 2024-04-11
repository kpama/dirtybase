use dirtybase_config::DirtyConfig;
use dirtybase_contract::http::RouterManager;

mod migration;

pub struct Extension;

#[dirtybase_contract::async_trait]
impl dirtybase_contract::ExtensionSetup for Extension {
    async fn setup(&self, _config: &DirtyConfig) {
        // event_handler::setup().await;
    }

    fn migrations(&self) -> dirtybase_contract::ExtensionMigrations {
        migration::setup()
    }

    async fn shutdown(&self) {
        println!("--- main application is shutting down -- ");
    }

    fn register_routes(&self, manager: RouterManager) -> RouterManager {
        manager
    }
}
