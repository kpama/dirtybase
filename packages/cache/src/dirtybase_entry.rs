mod migration;
use dirtybase_contract::{ExtensionSetup, app_contract::Context};

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, context: &Context) {
        super::setup(context).await;
    }
}
