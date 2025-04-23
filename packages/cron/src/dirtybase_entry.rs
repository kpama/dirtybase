use dirtybase_contract::{ExtensionSetup, app_contract::Context, cli_contract::CliCommandManager};

use crate::{config::CronConfig, register_resource_manager};

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, context: &Context) {
        if let Ok(config) = context.get_config::<CronConfig>("dty::cron").await {
            context.set(config).await;
        }

        register_resource_manager().await;
    }

    fn register_cli_commands(&self, manager: CliCommandManager) -> CliCommandManager {
        super::cli::setup_cli(manager)
    }
}
