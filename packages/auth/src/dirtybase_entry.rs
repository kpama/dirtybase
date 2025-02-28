mod migration;

use dirtybase_contract::{
    ExtensionMigrations, ExtensionSetup, config::DirtyConfig, http::WebMiddlewareManager,
};

use crate::{
    AuthConfig, middlewares::setup_middlewares, register_storages, setup_context_managers,
};

#[derive(Debug, Default)]
pub struct Extension {
    is_enable: bool,
}

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, base_config: &DirtyConfig) {
        let global_config = AuthConfig::from_dirty_config(base_config).await;
        self.is_enable = global_config.is_enabled();

        self.global_container().set_type(global_config).await;

        if !self.is_enable {
            tracing::debug!("Auth is not enabled");
            return;
        }

        register_storages().await;
        setup_context_managers().await;
    }

    fn migrations(&self) -> Option<ExtensionMigrations> {
        migration::setup()
    }

    fn register_web_middlewares(&self, manager: WebMiddlewareManager) -> WebMiddlewareManager {
        if !self.is_enable {
            return manager;
        }

        setup_middlewares(manager)
    }
}
