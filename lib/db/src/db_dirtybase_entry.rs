use dirtybase_contract::{dirtybase_config::DirtyConfig, ExtensionSetup};

pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&self, _config: &DirtyConfig) {
        super::setup_handlers().await;
    }
}
