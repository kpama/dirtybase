mod http;
mod middlewares;
pub mod migration;

use dirtybase_contract::{
    ExtensionMigrations, ExtensionSetup,
    app_contract::Context,
    auth_contract::Gate,
    http_contract::{RouterManager, WebMiddlewareManager},
};
use middlewares::setup_middlewares;

use crate::{AuthConfig, DATABASE_STORAGE, guards::register_guards, register_storages};

#[derive(Debug, Default)]
pub struct Extension {
    is_enable: bool,
    is_db_storage: bool,
}

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, global_context: &Context) {
        let global_config = global_context
            .get_config::<AuthConfig>("auth")
            .await
            .unwrap();

        self.is_enable = global_config.is_enabled();
        self.is_db_storage = global_config.storage_ref().as_str() == DATABASE_STORAGE;

        global_context
            .container()
            .resolver::<Gate>(|sc| {
                tracing::info!("calling the gate resolver: {}", sc.id());
                Box::pin(async {
                    //..
                    Gate::new(sc)
                })
            })
            .await;

        if !self.is_enable {
            tracing::debug!("Auth is not enabled");
            return;
        }

        self.global_container().set_type(global_config).await;

        register_storages().await;
        register_guards().await;
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

    fn register_routes(&self, manager: &mut RouterManager) {
        http::register_routes(manager)
    }
}
