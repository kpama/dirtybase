mod migration;
use dirtybase_contract::{ExtensionMigrations, ExtensionSetup, app_contract::Context};
use migration::setup;

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, context: &Context) {
        super::setup(context).await;
    }

    fn migrations(&self, _context: &Context) -> Option<ExtensionMigrations> {
        setup()
    }
}
