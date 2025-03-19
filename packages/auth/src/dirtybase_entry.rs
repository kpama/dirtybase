mod http;
mod middlewares;
mod migration;

use dirtybase_contract::{
    ExtensionMigrations, ExtensionSetup,
    app::Context,
    http::{RouterManager, WebMiddlewareManager},
    prelude::IntoResponse,
};
use dirtybase_helper::hash::sha256;
use middlewares::setup_middlewares;
use serde::{Deserialize, Serialize};

use crate::{AuthConfig, DATABASE_STORAGE, register_storages, setup_context_managers};

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

        if !self.is_enable {
            tracing::debug!("Auth is not enabled");
            return;
        }

        self.global_container().set_type(global_config).await;

        register_storages().await;
        setup_context_managers().await;
    }

    fn migrations(&self, _global_context: &Context) -> Option<ExtensionMigrations> {
        if !self.is_enable {
            return None;
        }

        if self.is_db_storage {
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

    fn register_routes(
        &self,
        manager: RouterManager,
        middleware_manager: &WebMiddlewareManager,
    ) -> RouterManager {
        http::register_routes(manager, middleware_manager)
    }
}
