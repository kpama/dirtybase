use axum::{
    body::Body,
    extract::Request,
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

use crate::{
    http_contract::{ModelBindResolver, api::ApiResponse},
    prelude::{Context, MiddlewareParam},
};

pub async fn handle_bind_middleware(
    req: Request,
    param: MiddlewareParam,
    next: Next,
) -> impl IntoResponse {
    if let Some(context) = req.extensions().get::<Context>().cloned() {
        let alias = param.kind();

        // Middleware added for a particular binding
        if !alias.is_empty() {
            return match ModelBindResolver::new(context.clone(), Some(param))
                .await
                .inject_alias(alias.clone().as_str())
                .await
            {
                Ok(false) => build_not_found_response(&req, &alias),
                _ => next.run(req).await,
            };
        } else {
            return match ModelBindResolver::new(context.clone(), Some(param))
                .await
                .inject_all_bindings()
                .await
            {
                Ok(false) => build_not_found_response(&req, &alias),
                _ => next.run(req).await,
            };
        }
    }
    next.run(req).await
}

// FIXME: We should be serving the appropriate response type based on the
//        the endpoint collection. Example: API returns with a JSON response
fn build_not_found_response(req: &Request, alias: &str) -> Response<Body> {
    if let Some(header) = req.headers().get("accept") {
        if let Ok(accept) = header.to_str() {
            if accept == "application/json" {
                return ApiResponse::<String>::error(anyhow::anyhow!("no binding for: {}", alias))
                    .into_response();
            }
        }
    }
    (StatusCode::NOT_FOUND, String::new()).into_response()
}
