use std::collections::HashMap;

use dirtybase_contract::{app::Context, http::prelude::*};

use super::jwt_auth_middleware::jwt_auth;

pub async fn handle_auth_middleware(
    mut req: Request,
    next: Next,
    params: Option<HashMap<String, String>>,
) -> impl IntoResponse {
    if params.is_none() {
        tracing::debug!("using session auth");
    } else {
        tracing::debug!(">>>>>>>>>>>>>>>>>>> In auth middleware: {:#?}", &params);
    }

    if let Some(p) = params {
        if p.contains_key("jwt") {
            // FIXME: pass the request and the storage provider to the specific auth
            let result = jwt_auth(req).await;
            req = result.0;
            if let Ok(Some(user)) = result.1 {
                let context = req.extensions().get::<Context>().unwrap();
                context.set(user).await;
                return next.run(req).await;
            }
        }
    }

    (StatusCode::UNAUTHORIZED, ()).into_response()
}
