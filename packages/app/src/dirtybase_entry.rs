use dirtybase_config::DirtyConfig;
use dirtybase_contract::{
    cli::CliCommandManager,
    http::{RouterManager, WebMiddlewareManager},
};

use crate::http;

mod commands_setup;
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
        log::info!("--- main application is shutting down -- ");
    }

    fn register_routes(
        &self,
        manager: RouterManager,
        _middleware: &WebMiddlewareManager,
    ) -> RouterManager {
        http::controllers::register(manager)
    }

    fn register_cli_commands(&self, manager: CliCommandManager) -> CliCommandManager {
        commands_setup::register(manager)
    }
}
