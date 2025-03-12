use dirtybase_contract::{ExtensionSetup, app::Context};

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, context: &Context) {
        super::setup(context).await;
    }
}
