use dirtybase_contract::{ExtensionSetup, app_contract::Context, cli_contract::CliCommandManager};

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, context: &Context) {
        super::setup(context).await;
    }

    fn register_cli_commands(&self, manager: CliCommandManager) -> CliCommandManager {
        super::cli::setup_cli(manager)
    }
}
