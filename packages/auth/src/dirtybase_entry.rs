mod migration;

use dirtybase_contract::{
    ExtensionMigrations, ExtensionSetup, app::Context, http::WebMiddlewareManager,
};

use crate::{
    AuthConfig, DATABASE_STORAGE, middlewares::setup_middlewares, register_storages,
    setup_context_managers,
};

#[derive(Debug, Default)]
pub struct Extension {
    is_enable: bool,
    is_db_storage: bool,
}

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, global_context: &Context) {
        let global_config = global_context
            .get_config::<AuthConfig>("dirtybase::auth")
            .await
            .unwrap();
        self.is_enable = global_config.is_enabled();
        self.is_db_storage = global_config.storage_ref().as_str() == DATABASE_STORAGE;

        self.global_container().set_type(global_config).await;

        if !self.is_enable {
            tracing::debug!("Auth is not enabled");
            return;
        }

        register_storages().await;
        setup_context_managers().await;
    }

    fn migrations(&self, _global_context: &Context) -> Option<ExtensionMigrations> {
        if !self.is_enable {
            return None;
        }
        if (self.is_db_storage) {
            return migration::setup();
        }

        None
    }

    fn register_web_middlewares(&self, manager: WebMiddlewareManager) -> WebMiddlewareManager {
        if !self.is_enable {
            return manager;
        }

        setup_middlewares(manager)
    }
}
