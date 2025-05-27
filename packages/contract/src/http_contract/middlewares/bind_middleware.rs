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

        return match ModelBindResolver::new(context.clone(), Some(param))
            .await
            .inject_alias(alias.clone().as_str())
            .await
        {
            Ok(_) => next.run(req).await,
            _ => (StatusCode::NOT_FOUND, String::new()).into_response(),
        };
    }

    println!(">>> binding: {:#?}", &param);
    next.run(req).await
}
