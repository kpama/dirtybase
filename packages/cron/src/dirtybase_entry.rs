use dirtybase_contract::{config::DirtyConfig, ExtensionSetup};

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, config: &DirtyConfig) {
        super::setup(config).await;
    }
}
