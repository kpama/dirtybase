use dirtybase_contract::{http::HttpContext, session::Session};

use crate::GuardResolver;

pub const SESSION_GUARD: &'static str = "session";

pub async fn authenticate(resolver: GuardResolver) -> GuardResolver {
    tracing::info!(">>>> In SESSION Authentication guard");
    if let Ok(session) = resolver.context_ref().get::<Session>().await {
        tracing::info!("session guard, session ID: {}", session.id());
        if let Ok(http_context) = resolver.context().get::<HttpContext>().await {
            //
            tracing::warn!(">>>> IP from http context: {:?}", http_context.ip())
        }
    }

    resolver
}
