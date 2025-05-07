use dirtybase_contract::{
    app_contract::{CtxExt, RequestContext},
    auth_contract::{AuthUser, AuthUserPayload, LoginCredential},
    axum::response::Html,
    http_contract::{HttpContext, api::ApiResponse, named_routes_axum, prelude::*},
    session_contract::Session,
};
use dirtybase_helper::{hash::sha256, security::random_bytes_hex};

use crate::{AuthConfig, StorageResolver};

pub(crate) async fn login_form_handler(
    RequestContext(context): RequestContext,
) -> impl IntoResponse {
    let mut submit_uri = named_routes_axum::helpers::get_path("auth:do-signin");
    if let Ok(auth_config) = context.get_config::<AuthConfig>("auth").await {
        submit_uri = named_routes_axum::helpers::get_path(&auth_config.auth_route());
    }

    Html(
            format!("<h1>Login Form</h1>
          <form method='post' action='{}'>
        <label>Username: </label><input type='text' name='username' placeholder='username' value='admin' /> <br/>
        <label>Password: </label><input type='password' name='password' placeholder='password' value='password' /> <br/>
        <button type='submit'>Login</button>
        <p>
             <a href='/auth/register-form'>Register </a>
        </p>
      </form>", submit_uri),
        )
}

pub(crate) async fn handle_login_request(
    RequestContext(ctx): RequestContext,
    CtxExt(http_ctx): CtxExt<HttpContext>,
    Form(cred): Form<LoginCredential>,
) -> Response<Body> {
    // TODO: This will use the auth service in the future
    let storage = StorageResolver::from_context(ctx.clone())
        .await
        .get_provider()
        .await
        .unwrap();
    let mut session = ctx.get::<Session>().await.unwrap();

    let result = if cred.username().is_some() {
        storage
            .find_by_username(cred.username().as_ref().unwrap())
            .await
    } else {
        let hash = sha256::hash_str(&cred.email().cloned().unwrap_or(":::nothing:::".to_string()));
        storage.find_by_email_hash(&hash).await
    };
    if let Ok(Some(user)) = result {
        if user.verify_password(cred.password()) {
            // 1. generate cookie id
            let cookie_key = random_bytes_hex(4);
            // 2. generate auth hash
            let hash = random_bytes_hex(16);
            // 3. store hash in the session and cookie
            let previous_path = session
                .get::<String>("_auth_prev_path")
                .await
                .unwrap_or_default();
            session = session.invalidate().await;
            ctx.set(session.clone()).await;
            session.put("auth_hash", &hash).await;
            session.put("auth_cookie_key", &cookie_key).await;
            session.put("auth_user_id", user.id()).await;
            let cookie = session.make_session_cookie(&cookie_key, hash); // FIXME: Build the cookie instance!!!!
            http_ctx.set_cookie(cookie).await;

            let bdy = Body::empty();

            let mut response = bdy.into_response();

            response.headers_mut().append(
                header::LOCATION,
                header::HeaderValue::from_str(&previous_path).unwrap(),
            );
            *response.status_mut() = StatusCode::SEE_OTHER;
            return response;

            // return Html(format!(
            //     "Welcome: {}, session id in cookie",
            //     user.username_ref(),
            // ));
        }
    }

    // Html("Auth failed".to_string()).into_response()
    // Response::new(Body::from(Html("".to_string())))
    let bdy = Body::empty();
    bdy.into_response()
}

pub(crate) async fn handle_get_auth_token(
    RequestContext(ctx): RequestContext,
    Json(cred): Json<LoginCredential>,
) -> impl IntoResponse {
    // TODO: This will use the auth service in the future
    let storage = StorageResolver::from_context(ctx)
        .await
        .get_provider()
        .await
        .unwrap();

    let result = if cred.username().is_some() {
        storage
            .find_by_username(cred.username().as_ref().unwrap())
            .await
    } else {
        let hash = sha256::hash_str(&cred.email().cloned().unwrap_or(":::nothing:::".to_string()));
        storage.find_by_email_hash(&hash).await
    };

    let mut res = ApiResponse::<String>::default();

    if let Ok(Some(user)) = result {
        if user.verify_password(cred.password()) {
            if let Some(token) = user.generate_token() {
                res.set_data(token);
            }
        }
    }

    if !res.has_data() {
        res.set_error("authentication failed");
    }

    res
}

pub(crate) async fn register_form_handler() -> impl IntoResponse {
    Html(
        "<h1>Register Form</h1><form method='post' action='/auth/do-registration'>
    <label>Username: </label><input type='text' name='username' placeholder='username' /> <br/>
    <label>Email: </label><input type='text' name='email' placeholder='email' /> <br/>
    <label>Password: </label><input type='password' name='password' placeholder='password' /> <br/>
    <label>Confirm Password: </label><input type='password' name='confirm_password' placeholder='password' /> <br/>
    <button type='submit'>Register</button>
  </form>",
    )
}

pub(crate) async fn handle_register_request(
    RequestContext(ctx): RequestContext,
    Form(mut payload): Form<AuthUserPayload>,
) -> impl IntoResponse {
    // FIXME: This will use the auth service in the future
    let storage = StorageResolver::from_context(ctx)
        .await
        .get_provider()
        .await
        .unwrap();

    payload.rotate_salt = true;
    payload.status = match payload.status {
        Some(s) => Some(s),
        None => Some(dirtybase_contract::auth_contract::AuthUserStatus::Pending),
    };

    // FIXME: Send email for verification
    payload.verified_at = Some(dirtybase_helper::time::current_datetime());

    let mut token = String::new();
    if let Ok(user) = storage.store(payload).await {
        match user.generate_token() {
            Some(t) => token = t,
            None => {
                tracing::error!("did not get back user token: {:?}", user.id())
            }
        }
    }

    format!("token: {}", token)
}

pub(crate) async fn handle_api_register_request(
    RequestContext(ctx): RequestContext,
    Json(mut payload): Json<AuthUserPayload>,
) -> ApiResponse<String> {
    // This will use the auth service in the future
    let storage = StorageResolver::from_context(ctx)
        .await
        .get_provider()
        .await
        .unwrap();

    payload.rotate_salt = true;
    let mut resp = ApiResponse::<String>::default();

    if let Ok(user) = storage.store(payload).await {
        resp.set_data(user.generate_token().unwrap());
    } else {
        resp.set_error("could not register user");
    }
    resp
}

pub(crate) async fn handle_api_get_me(
    RequestContext(context): RequestContext,
) -> ApiResponse<AuthUser> {
    // FIXME: Get the auth user another way
    if let Ok(user) = context.get::<AuthUser>().await {
        ApiResponse::success(user)
    } else {
        ApiResponse::error("user not found")
    }
}
