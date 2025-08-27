use dirtybase_contract::{ExtensionSetup, app_contract::Context, cli_contract::CliCommandManager};

use crate::{config::CronConfig, register_resource_manager};

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, context: &Context) {
        _ = context
            .get_config_once::<CronConfig>("cron")
            .await
            .expect("could not load cron configuration");

        register_resource_manager().await;
    }

    fn register_cli_commands(&self, manager: CliCommandManager) -> CliCommandManager {
        super::cli::setup_cli(manager)
    }
}
