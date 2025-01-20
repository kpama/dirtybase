use axum_extra::extract::CookieJar;
use dirtybase_contract::{
    app::CtxExt,
    auth::{UserProviderService, UserProviderTrait},
    http::prelude::*,
};
use serde::Deserialize;

pub struct NormalAuthMiddleware;

impl WebMiddlewareRegisterer for NormalAuthMiddleware {
    fn register(&self, router: WrappedRouter) -> WrappedRouter {
        println!("registering normal auth middleware");
        router.middleware(handle_normal_auth_middleware)
    }
}

async fn handle_normal_auth_middleware(req: Request, next: Next) -> impl IntoResponse {
    // 1. Check if there is an active session
    // 1.1 Check if there is a seesion ID in the request header
    // 1.2. If there is one, use it to check if the current session
    //      has been authenticated
    // 1.3 If so, allow the request
    // 1.4. Else try plucking the username and password
    println!(">>>> Normal auth ran <<<<<");
    next.run(req).await
}

pub async fn handle_user_login_request(
    _jar: CookieJar,
    CtxExt(user_provider): CtxExt<UserProviderService>,
    Form(form): Form<UserCredential>,
) -> impl IntoResponse {
    if !form.username.is_empty() {
        let username = form.username.clone();
        let hash_pasword = user_provider.by_username(&username).await;

        if authenticate_from_request(form, hash_pasword).await {
            return "You successfully logged".to_string();
        }
    }

    "Login authentication failed".to_string()
}

pub async fn authenticate_from_request(credential: UserCredential, hash_password: String) -> bool {
    println!("login form: {:#?}", &credential);
    let request_pws_hash = credential.password.to_string(); // hash the raw password

    request_pws_hash == hash_password
}

#[derive(Debug, Deserialize)]
pub struct UserCredential {
    // #[serde(default)]
    // to: String,
    username: String,
    password: String,
}
