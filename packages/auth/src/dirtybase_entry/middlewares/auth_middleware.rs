use std::collections::HashMap;

use dirtybase_contract::{app::Context, http::prelude::*};

use crate::{GuardResolver, StorageResolver, guards::session_guard::SESSION_GUARD};

pub async fn handle_auth_middleware(
    req: Request,
    next: Next,
    params: Option<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(mut p) = params {
        // changes jwt => "" to guard => "jwt"
        if p.keys().len() == 1 && p.get("guard").is_none() {
            let first = p.keys().into_iter().next().cloned().unwrap();
            p.insert("guard".to_string(), first);
        }

        let guard_name = p
            .remove("guard")
            .unwrap_or_else(|| SESSION_GUARD.to_string());
        let context = req
            .extensions()
            .get::<Context>()
            .cloned()
            .unwrap_or_default();

        tracing::error!("current guard name: {}", &guard_name);

        if let Some(storage) = StorageResolver::new(context.clone()).get_provider().await {
            let guard = GuardResolver::new(req, storage, &guard_name).guard().await;
            if let Some(Ok(Some(user))) = guard.user {
                context.set(user).await;
                return next.run(guard.req).await;
            }
            if let Some(response) = guard.resp {
                return response;
            }

            println!(">>>>>>>>>>>>>>>. here <<<<<<<<<<<l");
            if let Some(Err(err)) = guard.user {
                tracing::error!("authentication error: {}", err.to_string());
            }
        }
    }

    (StatusCode::UNAUTHORIZED, ()).into_response()
}
