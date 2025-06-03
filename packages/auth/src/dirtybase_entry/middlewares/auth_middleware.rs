use dirtybase_contract::{
    app_contract::Context,
    auth_contract::{GuardResolver, GuardResponse, StorageResolver},
    http_contract::prelude::*,
};

use crate::{AuthConfig, guards::session_guard::SESSION_GUARD};

pub async fn handle_auth_middleware(
    req: Request,
    param: MiddlewareParam,
    next: Next,
) -> impl IntoResponse {
    let mut guard_name = param.kind_ref().to_string();

    if guard_name.is_empty() {
        guard_name = SESSION_GUARD.to_string()
    }

    let context = req
        .extensions()
        .get::<Context>()
        .cloned()
        .unwrap_or_default();
    tracing::debug!("current auth guard: {}", &guard_name);

    if let Ok(config) = context.get_config::<AuthConfig>("auth").await {
        if let Some(storage) = StorageResolver::from_context(context.clone())
            .await
            .get_provider(config.storage_ref())
            .await
        {
            let result =
                GuardResolver::new(req.headers().clone(), context.clone(), storage.clone())
                    .guard(&guard_name)
                    .await;

            if !result.is_success() {
                return result.response().unwrap_or_else(|| {
                    //
                    GuardResponse::unauthorized().response().unwrap()
                });
            }

            return next.run(req).await;
        }
    }

    (StatusCode::UNAUTHORIZED, ()).into_response()
}
