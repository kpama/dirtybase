//! App's Common Language Expression Observers

use dirtybase_contract::{
    http_contract::HttpContext,
    prelude::{Observable, observable::cel::CelCoreVariable},
};

use crate::app::AppService;

pub(crate) async fn register_observers() {
    CelCoreVariable::subscribe(|mut core, ctx| async move {
        let app = ctx
            .get::<AppService>()
            .await
            .expect("could not get app service");

        // Setup defaults
        _ = core.add_to_environment("is_web", false);
        _ = core.add_to_environment("app_name", app.config_ref().app_name());
        _ = core.add_to_environment("env", app.config_ref().environment());

        // Add HTTP data if this is web request
        if let Ok(http_ctx) = ctx.get::<HttpContext>().await {
            _ = core.add_to_environment("is_web", true);
            _ = core.add_to_environment("client_ip", http_ctx.ip());
            _ = core.add_to_environment("domain", http_ctx.domain());
            _ = core.add_to_environment("web_queries", http_ctx.query_as_map());
            _ = core.add_to_environment("web_path", http_ctx.path());
        }

        core
    })
    .await;
}
