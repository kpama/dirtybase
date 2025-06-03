use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse};

use crate::{
    http_contract::ModelBindResolver,
    prelude::{Context, MiddlewareParam},
};

pub async fn handle_bind_middleware(
    req: Request,
    param: MiddlewareParam,
    next: Next,
) -> impl IntoResponse {
    // ....
    if let Some(context) = req.extensions().get::<Context>().cloned() {
        let alias = param.kind();

        // middleware added for a particular binding
        if !alias.is_empty() {
            return match ModelBindResolver::new(context.clone(), Some(param))
                .await
                .inject_alias(alias.clone().as_str())
                .await
            {
                Ok(false) => (StatusCode::NOT_FOUND, String::new()).into_response(),
                _ => next.run(req).await,
            };
        } else {
            return match ModelBindResolver::new(context.clone(), Some(param))
                .await
                .inject_all_bindings()
                .await
            {
                Ok(false) => (StatusCode::NOT_FOUND, String::new()).into_response(),
                _ => next.run(req).await,
            };
        }
    }

    println!(">>> binding: {:#?}", &param);
    next.run(req).await
}
