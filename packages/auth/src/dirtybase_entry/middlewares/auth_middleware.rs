use dirtybase_contract::{app_contract::Context, http_contract::prelude::*};

use crate::{GuardResolver, StorageResolver, guards::session_guard::SESSION_GUARD};

pub async fn handle_auth_middleware(
    req: Request,
    param: MiddlewareParam,
    next: Next,
) -> impl IntoResponse {
    let mut guard_name = param.kind_ref().to_string();

    if guard_name.is_empty() {
        guard_name = SESSION_GUARD.to_string()
    }

    let context = if let Some(ctx) = req.extensions().get::<Context>().cloned() {
        ctx
    } else {
        Context::default()
    };

    tracing::debug!("current auth guard: {}", &guard_name);

    if let Some(storage) = StorageResolver::from_context(context.clone())
        .await
        .get_provider()
        .await
    {
        let guard = GuardResolver::new(req, storage, &guard_name).guard().await;
        if let Some(Ok(Some(user))) = guard.user {
            context.set(user).await;
            return next.run(guard.req).await;
        }
        if let Some(response) = guard.resp {
            return response;
        }

        if let Some(Err(err)) = guard.user {
            tracing::error!("authentication error: {}", err.to_string());
        }
    }

    (StatusCode::UNAUTHORIZED, ()).into_response()
}
