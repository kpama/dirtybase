use dirtybase_contract::{
    app::Context,
    cli::CliCommandManager,
    config::DirtyConfig,
    http::{RouterManager, WebMiddlewareManager},
};

mod commands_setup;

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl dirtybase_contract::ExtensionSetup for Extension {
    async fn setup(&mut self, _context: &Context) {
        // event_handler::setup().await;
    }

    fn migrations(&self, context: &Context) -> Option<dirtybase_contract::ExtensionMigrations> {
        None
    }

    async fn shutdown(&mut self, _context: &Context) {
        log::info!("--- main application is shutting down -- ");
    }

    fn register_routes(
        &self,
        manager: RouterManager,
        _middleware: &WebMiddlewareManager,
    ) -> RouterManager {
        manager
    }

    fn register_cli_commands(&self, manager: CliCommandManager) -> CliCommandManager {
        commands_setup::register(manager)
    }
}
