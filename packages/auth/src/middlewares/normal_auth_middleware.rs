use axum_extra::extract::CookieJar;
use dirtybase_contract::{
    app::{Context, CtxExt, UserContext},
    auth::{UserProviderService, UserProviderTrait},
    http::prelude::*,
    session::Session,
    ExtensionManager,
};
use serde::Deserialize;

use crate::{AuthConfig, AuthManager};

pub async fn handle_normal_auth_middleware(req: Request, next: Next) -> impl IntoResponse {
    let context;
    let session;
    let mut is_success = false;

    if let Some(c) = req.extensions().get::<Context>() {
        context = c;
    } else {
        tracing::error!("context not set on request");
        return return_500_response();
    }

    let manager = context.get::<AuthManager>().await.unwrap();

    if !manager.is_enable() {
        tracing::warn!("auth extension is disabled");
        return next.run(req).await;
    }

    // 1. Check if there is an active session
    if let Some(s) = context.get::<Session>().await {
        session = s;
    } else {
        tracing::error!("session not set on request");
        return return_500_response();
    }

    tracing::error!(
        "jwt token from session: {:?}",
        session.get::<String>("auth_jwt").await
    );

    // 1.2. If there is one, use it to check if the current session
    //      has been authenticated
    // 1.3 If so, allow the request
    // 1.4. Else try plucking the username and password

    if is_success {
        next.run(req).await
    } else {
        log::warn!("request is forbidden");
        return_403_response()
    }
}

pub fn return_403_response() -> Response<Body> {
    (StatusCode::FORBIDDEN, ()).into_response()
}
pub fn return_500_response() -> Response<Body> {
    (StatusCode::INTERNAL_SERVER_ERROR, ()).into_response()
}

pub async fn handle_user_login_web_request(
    jar: CookieJar,
    CtxExt(user_provider): CtxExt<UserProviderService>,
    Extension(ctx): Extension<Context>,
    Form(form): Form<UserCredential>,
) -> impl IntoResponse {
    if !form.username.is_empty() {
        let username = form.username.clone();
        let hash_password = user_provider.by_username(&username).await;

        if authenticate_from_request(form, hash_password).await {
            return "You successfully logged".to_string();
        }
    }

    "Login authentication failed".to_string()
}

// pub async fn handle_user_login_request(
//     credential: UserCredential,
//     session: &SessionManager,
//     user_provider: &UserProviderService,
// ) -> bool {
//     // FIXME: This is a work in progress
//     true
// }

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
