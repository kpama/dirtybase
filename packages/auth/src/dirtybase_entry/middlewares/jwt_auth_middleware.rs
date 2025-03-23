use dirtybase_contract::{
    app::Context,
    auth::{AuthUser, ParseToken, StorageResolverPipeline},
    http::prelude::*,
};

pub async fn jwt_auth(req: Request) -> (Request, Result<Option<AuthUser>, anyhow::Error>) {
    let context = req.extensions().get::<Context>().unwrap();
    if let Some(header) = req.headers().get("authorization") {
        let bearer = axum_extra::headers::authorization::Bearer::decode(header);
        if let Some(cred) = bearer {
            let token = cred.token().to_string();
            if token.contains("|") {
                if let Ok(token) = ParseToken::try_from(token) {
                    let storage = StorageResolverPipeline::new(context.clone())
                        .get_provider()
                        .await
                        .unwrap();

                    return (req, storage.find_by_id(token.id()).await);
                }
            }
        }
    }

    (req, Ok(None))
}

pub async fn handle_jwt_auth_middleware(req: Request, next: Next) -> impl IntoResponse {
    println!(">>>> jwt auth ran <<<<");
    let context = req.extensions().get::<Context>().unwrap();

    if let Some(_user) = context.user().await {
        // FIXME: Check the the user is actually log in
        return next.run(req).await;
    }

    if let Some(header) = req.headers().get("authorization") {
        let bearer = axum_extra::headers::authorization::Bearer::decode(header);
        if let Some(cred) = bearer {
            let token = cred.token().to_string();
            if token.contains("|") {
                if let Ok(token) = ParseToken::try_from(token) {
                    let storage = StorageResolverPipeline::new(context.clone())
                        .get_provider()
                        .await
                        .unwrap();

                    if let Ok(Some(existing)) = storage.find_by_id(token.id()).await {
                        tracing::debug!("current user: {}", existing);
                        return next.run(req).await;
                    }
                }
            }
        }
    }

    (StatusCode::FORBIDDEN, ()).into_response()
}
