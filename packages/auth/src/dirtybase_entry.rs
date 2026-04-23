mod http;
mod middlewares;
mod migration;
mod observer;
mod seeder;

use dirtybase_contract::{
    ExtensionMigrations, ExtensionSetup, app_contract::Context, auth_contract::Gate,
    http_contract::RouterManager, prelude::ArgMatches,
};

use crate::{AuthConfig, guards::register_guards, register_storages, storage2};

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
        self.allow_self_signup = global_config.allow_self_signup();
        self.is_db_storage = global_config.is_db_storage();


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

        storage2::register_storage().await;
        storage2::register_manager().await;

        register_storages().await;
        register_guards().await;

        observer::register_observers().await;
    }

    fn migrations(&self, _: &Context) -> Option<ExtensionMigrations> {
        if self.is_db_storage && self.is_enable {
            return migration::setup();
        }

        None
    }

    fn register_routes(&self, manager: &mut RouterManager) {
        http::register_routes(manager, self.allow_self_signup)
    }

    async fn on_cli_command(
        &self,
        cmd: &str,
        matches: ArgMatches,
        _context: Context,
    ) -> ArgMatches {
        // TODO: Check the feature's flag
        if cmd == "seed" {
            #[cfg(feature = "seeders")]
            seeder::register_seeders().await;
        }

        matches
    }
}

impl AuthExtension {
    pub async fn config_from_ctx(ctx: &Context) -> Result<AuthConfig, anyhow::Error> {
        let config = ctx.get_config_once("auth").await;

        if config.is_err() {
            tracing::error!("could not fetch auth config: {:?}", config.as_ref().err());
        }

        config
    }
}
