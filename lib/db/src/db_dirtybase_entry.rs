use dirtybase_config::DirtyConfig;
use dirtybase_contract::ExtensionSetup;

pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&self, _config: &DirtyConfig) {
        super::setup_handlers().await;
    }
}
