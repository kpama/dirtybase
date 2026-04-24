use dirtybase_contract::{ExtensionSetup, app_contract::Context, cli_contract::CliCommandManager};

use crate::{config::CronConfig, register_resource_manager};

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, context: &Context) {
        _ = Self::config_from_ctx(context).await;
        register_resource_manager().await;
    }

    async fn on_new_context(&self, _context: &Context) {
        //
    }

    async fn register_cli_commands(&self, manager: CliCommandManager) -> CliCommandManager {
        super::cli::setup_cli(manager)
    }
}

impl Extension {
    pub async fn config_from_ctx(ctx: &Context) -> Result<CronConfig, anyhow::Error> {
        let result = ctx.get_config_once::<CronConfig>("cron").await;
        Ok(result.expect("could not load cron configuration"))
    }
}
