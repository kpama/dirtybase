mod migration;

use dirtybase_contract::{
    ExtensionMigrations, ExtensionSetup, app_contract::Context, async_trait,
    axum::response::Response, prelude::axum_extra::extract::CookieJar,
};

use crate::{
    SessionConfig,
    resource_manager::register_resource_manager,
    session_resolver::{attach_session_cookie, register_session_resolver},
};

#[derive(Default)]
pub struct SessionExtension;

#[async_trait]
impl ExtensionSetup for SessionExtension {
    async fn setup(&mut self, ctx: &Context) {
        if let Ok(config) = Self::config_from_ctx(&ctx).await {
            ctx.set(config).await;
        }

        register_resource_manager().await;
        register_session_resolver(&ctx).await;
    }

    async fn on_web_response(
        &self,
        resp: Response,
        cookie: CookieJar,
        context: Context,
    ) -> (Response, CookieJar) {
        (resp, attach_session_cookie(&context, cookie).await)
    }

    fn migrations(&self, _: &Context) -> Option<ExtensionMigrations> {
        migration::setup()
    }
}

impl SessionExtension {
    pub async fn config_from_ctx(ctx: &Context) -> Result<SessionConfig, anyhow::Error> {
        let result = ctx.get_config::<SessionConfig>("session").await;

        if result.is_err() {
            tracing::error!("could not load session config: {:?}", result.as_ref().err());
        }

        result
    }
}
