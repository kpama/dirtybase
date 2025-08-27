use dirtybase_contract::{
    app_contract::Context,
    auth_contract::{GuardResolver, GuardResponse, StorageResolver},
    http_contract::prelude::*,
};

use crate::{AuthExtension, guards::session_guard::SESSION_GUARD};

pub async fn handle_auth_middleware(
    req: Request,
    param: MiddlewareParam,
    next: Next,
) -> impl IntoResponse {
    let mut guard_name = param.kind_ref();

    if guard_name.is_empty() {
        guard_name = SESSION_GUARD
    }

    let Some(context) = req.extensions().get::<Context>().cloned() else {
        tracing::error!(target = "auth middleware", "could not get context");
        return (StatusCode::UNAUTHORIZED, ()).into_response();
    };
    tracing::debug!("current auth guard: {}", guard_name);

    if let Ok(config) = AuthExtension::config_from_ctx(&context).await {
        if let Some(storage) = StorageResolver::from_context(context.clone())
            .await
            .get_provider(config.storage_ref())
            .await
        {
            let result =
                GuardResolver::new(req.headers().clone(), context.clone(), storage.clone())
                    .guard(guard_name)
                    .await;

            if !result.is_success() {
                return result.response().unwrap_or_else(|| {
                    GuardResponse::unauthorized().response().unwrap() // NOTE: unwrap is okay here
                });
            }

            return next.run(req).await;
        }
    }

    (StatusCode::UNAUTHORIZED, ()).into_response()
}
