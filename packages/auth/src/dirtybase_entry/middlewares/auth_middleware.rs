use std::collections::HashMap;

use dirtybase_contract::{app::Context, http::prelude::*};

use crate::{GuardResolver, StorageResolver};

pub async fn handle_auth_middleware(
    mut req: Request,
    next: Next,
    mut params: Option<HashMap<String, String>>,
) -> impl IntoResponse {
    if params.is_none() {
        tracing::debug!("using session auth");
    } else {
        tracing::debug!(">>>>>>>>>>>>>>>>>>> In auth middleware: {:#?}", &params);
    }

    if let Some(mut p) = params {
        let guard_name = p.remove("guard").unwrap_or_else(|| "normal".to_string());
        let context = req
            .extensions()
            .get::<Context>()
            .cloned()
            .unwrap_or_default();
        if let Some(storage) = StorageResolver::new(context.clone()).get_provider().await {
            let mut guard = GuardResolver::new(req, storage, &guard_name).guard().await;
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
    }

    (StatusCode::UNAUTHORIZED, ()).into_response()
}
