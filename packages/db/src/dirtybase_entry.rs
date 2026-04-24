use dirtybase_contract::{ExtensionSetup, app_contract::Context, cli_contract::CliCommandManager};

use crate::{command::setup_commands, resource_manager::register_resource_manager};

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, _context: &Context) {
        register_resource_manager().await;
    }

    async fn on_new_context(&self, _context: &Context) {
        //
    }

    async fn register_cli_commands(&self, manager: CliCommandManager) -> CliCommandManager {
        setup_commands(manager)
    }
}
