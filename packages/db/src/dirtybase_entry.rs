use dirtybase_contract::{ExtensionSetup, app::Context, cli::CliCommandManager, prelude::Command};

use crate::{command::setup_commands, resource_manager::register_resource_manager};

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, _context: &Context) {
        super::setup_handlers().await;
        register_resource_manager().await;
    }

    fn register_cli_commands(&self, manager: CliCommandManager) -> CliCommandManager {
        setup_commands(manager)
    }
}
