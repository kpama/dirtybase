mod http;
mod manager;
mod migration;
mod model;
mod resource_manager;

pub mod storage;

use dirtybase_contract::prelude::*;

use crate::{
    MultitenantConfig, dirtybase_entry::resource_manager::register_multitenant_resource_manager,
};

pub use manager::*;

#[derive(Default)]
pub struct Extension {
    config: MultitenantConfig,
}

#[dirtybase_contract::async_trait]
impl dirtybase_contract::ExtensionSetup for Extension {
    async fn setup(&mut self, context: &Context) {
        self.config = context
            .get_config_once::<MultitenantConfig>("multitenant")
            .await
            .expect("could not load multi tenant configuration");
        register_multitenant_resource_manager().await;
    }

    fn migrations(&self, _context: &Context) -> Option<dirtybase_contract::ExtensionMigrations> {
        migration::setup()
    }
}
