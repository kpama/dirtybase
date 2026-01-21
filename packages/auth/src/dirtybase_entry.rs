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

use crate::{AuthConfig, guards::register_guards, register_storages, storage};

#[derive(Debug, Default)]
pub struct AuthExtension {
    is_enable: bool,
    is_db_storage: bool,
    allow_self_signup: bool,
}

#[dirtybase_contract::async_trait]
impl ExtensionSetup for AuthExtension {
    async fn setup(&mut self, ctx: &Context) {
        let global_config = Self::config_from_ctx(ctx)
            .await
            .expect("could not load auth config");

        self.is_enable = global_config.is_enabled();
        self.is_db_storage = global_config.storage_ref().as_str()
            == storage::database_storage::AuthUserDatabaseStorage::NAME;
        self.allow_self_signup = global_config.allow_self_signup();

        ctx.container()
            .resolver(|sc| async move {
                tracing::info!("called the gate resolver: {}", sc.id());
                Gate::new(sc)
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

    fn migrations(&self, _: &Context) -> Option<ExtensionMigrations> {
        if self.is_db_storage && self.is_enable {
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
        http::register_routes(manager, self.allow_self_signup)
    }
}

impl AuthExtension {
    pub async fn config_from_ctx(ctx: &Context) -> Result<AuthConfig, anyhow::Error> {
        let config = ctx.get_config("auth").await;

        if config.is_err() {
            tracing::error!("could not fetch auth config: {:?}", config.as_ref().err());
        }

        config
    }
}
