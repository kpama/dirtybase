use dirtybase_contract::prelude::*;

mod commands_setup;
mod observer;

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl dirtybase_contract::ExtensionSetup for Extension {
    async fn setup(&mut self, _: &Context) {
        // --

        observer::register_observers().await;
    }

    fn register_cli_commands(&self, manager: CliCommandManager) -> CliCommandManager {
        commands_setup::register(manager)
    }

    fn register_web_middlewares(&self, manager: WebMiddlewareManager) -> WebMiddlewareManager {
        dirtybase_contract::http_contract::middlewares::setup_middlewares(manager)
    }
}
