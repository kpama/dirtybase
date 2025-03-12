use dirtybase_contract::{ExtensionSetup, app::Context};

use crate::resource_manager::register_resource_manager;

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, _context: &Context) {
        super::setup_handlers().await;
        register_resource_manager().await;
    }
}
