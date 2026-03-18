use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse};

use crate::{
    auth_contract::{GuardResolver, GuardResponse, storage::PermStorageProvider},
    prelude::{Context, MiddlewareParam},
};

const AUTH_MIDDLEWARE_LOG: &str = "auth_contract_middleware";

pub const SESSION_GUARD: &str = "session";

pub async fn handle_auth_middleware(
    req: Request,
    param: MiddlewareParam,
    next: Next,
) -> impl IntoResponse {
    let mut guard_name = param.kind_ref();

    // NOTE: Fallback to the session guard if a guard is not specified
    if guard_name.is_empty() {
        guard_name = SESSION_GUARD
    }

    tracing::debug!(
        target = AUTH_MIDDLEWARE_LOG,
        "current auth guard: {}",
        guard_name
    );
    let Some(context) = req.extensions().get::<Context>().cloned() else {
        tracing::error!(target = AUTH_MIDDLEWARE_LOG, "could not get context");
        return (StatusCode::UNAUTHORIZED, ()).into_response();
    };

    let storage = if let Ok(s) = context.get::<PermStorageProvider>().await {
        s
    } else {
        return (StatusCode::SERVICE_UNAVAILABLE, ()).into_response();
    };

    let mut guard_response = GuardResolver::new(req.headers().clone(), context.clone(), storage)
        .guard(guard_name)
        .await;

    if !guard_response.is_success() && !guard_response.has_response() {
        guard_response.set_response(
            GuardResponse::unauthorized().response().unwrap(), // NOTE: unwrap is okay here
        );
    }

    return if guard_response.has_response() {
        tracing::trace!(target = AUTH_MIDDLEWARE_LOG, "serving guard response");
        guard_response
            .response()
            .expect("could not get guard returned response")
    } else {
        next.run(req).await
    };
}
