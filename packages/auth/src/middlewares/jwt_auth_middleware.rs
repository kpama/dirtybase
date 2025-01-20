use dirtybase_contract::{app::Context, http::prelude::*};

pub struct JWTAuthMiddleware;

impl WebMiddlewareRegisterer for JWTAuthMiddleware {
    fn register(
        &self,
        router: dirtybase_contract::http::WrappedRouter,
    ) -> dirtybase_contract::http::WrappedRouter {
        println!("registering jwt auth middleware");
        router.middleware(handle_jwt_auth_middleware)
    }
}

async fn handle_jwt_auth_middleware(req: Request, next: Next) -> impl IntoResponse {
    println!(">>>> jwt auth ran <<<<");

    if let Some(context) = req.extensions().get::<Context>() {
        if let Some(_user) = context.user() {
            // FIXME: Check the the user is actually log in
            return next.run(req).await;
        }
    }

    if let Some(header) = req.headers().get("authorization") {
        let bearer = axum_extra::headers::authorization::Bearer::decode(header);
        if let Some(cred) = bearer {
            println!("jwt token: {}", cred.token());
        }
    }
    next.run(req).await
}
