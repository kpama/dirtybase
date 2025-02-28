use std::sync::Arc;

use dirtybase_contract::{
    ExtensionSetup,
    app::{Context, ContextManager},
    config::DirtyConfig,
    db::base::manager::Manager,
    prelude::{ArgMatches, Request, axum_extra::extract::CookieJar},
};

use crate::{config::BaseConfig, connection_bus::MakePoolManagerCommand};

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, _config: &DirtyConfig) {
        super::setup_handlers().await;
        self.global_container()
            .set(ContextManager::<Manager>::new())
            .await;
    }

    async fn on_web_request(&self, req: Request, context: Context, _cookie: &CookieJar) -> Request {
        self.setup_context_connection(&context).await;
        req
    }

    async fn on_cli_command(
        &self,
        _cmd: &str,
        matches: ArgMatches,
        context: Context,
    ) -> ArgMatches {
        self.setup_context_connection(&context).await;
        matches
    }
}

impl Extension {
    async fn setup_context_connection(&self, context: &Context) {
        let tenant = context.tenant().await;
        let ctx = context.clone();
        let dirty_config = ctx.container().get::<DirtyConfig>().await.unwrap();

        context
            .container()
            .resolver(move |container| {
                let tenant = tenant.clone().unwrap_or_default();
                let dirty_config = dirty_config.clone();

                Box::pin(async move {
                    let config = tenant
                        .clone()
                        .config_to::<BaseConfig>("database")
                        .unwrap_or_else(|| BaseConfig::default());
                    let id = tenant.id().to_string();
                    let seconds = if tenant.is_global() { 15 } else { 5 };

                    container
                        .get::<ContextManager<Manager>>()
                        .await
                        .unwrap()
                        .context(
                            id.as_str(),
                            seconds,
                            || {
                                let dirty_config = dirty_config.clone();
                                Box::pin(async move {
                                    MakePoolManagerCommand::make(
                                        BaseConfig::set_from(&dirty_config).await,
                                    )
                                    .await
                                    .expect("could not create a database manager")
                                })
                            },
                            move |manager| {
                                tracing::trace!(
                                    "closing {} pool, it has been idle for {} seconds",
                                    manager.db_kind().as_str(),
                                    seconds
                                );
                                Box::pin(async move {
                                    manager.close().await;
                                })
                            },
                        )
                        .await
                })
            })
            .await;
    }
}
