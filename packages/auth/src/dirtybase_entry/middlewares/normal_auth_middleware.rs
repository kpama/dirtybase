use axum_extra::extract::CookieJar;
use dirtybase_contract::{
    app_contract::{Context, CtxExt},
    auth_contract::LoginCredential,
    http_contract::prelude::*,
    session_contract::Session,
    user::{UserProviderService, model::UserRepositoryTrait},
};
use serde::Deserialize;

use crate::StorageResolver;

pub async fn handle_normal_auth_middleware(req: Request, next: Next) -> impl IntoResponse {
    let context;
    let session;
    let is_success = false;

    if let Some(c) = req.extensions().get::<Context>() {
        context = c;
    } else {
        tracing::error!("context not set on request");
        return return_500_response();
    }

    let storage = StorageResolver::new(context.clone()).get_provider().await;
    tracing::error!("do we have auth user storage: {}", storage.is_some());

    // 1. Check if there is an active session
    if let Ok(s) = context.get::<Session>().await {
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
    credential: LoginCredential,
) -> impl IntoResponse {
    let user = if credential.username().is_none() {
        user_provider
            .find_by_username(credential.username().as_ref().unwrap(), false)
            .await
    } else {
        user_provider
            .find_by_email(credential.email().as_ref().unwrap(), false)
            .await
    };

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
    username: String,
    password: String,
}
