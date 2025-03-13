use dirtybase_contract::{
    cli::CliCommandManager,
    http::{RouterManager, WebMiddlewareManager},
};

mod commands_setup;

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl dirtybase_contract::ExtensionSetup for Extension {
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
