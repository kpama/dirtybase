use dirtybase_contract::{config::DirtyConfig, ExtensionSetup};

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, _config: &DirtyConfig) {
        super::setup_handlers().await;
    }
}
