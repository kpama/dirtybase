use dirtybase_contract::{
    app_contract::{CtxExt, RequestContext},
    auth_contract::{AuthUser, AuthUserPayload, AuthUserStorageProvider, LoginCredential},
    axum::response::Html,
    db_contract::types::ArcUuid7,
    http_contract::{HttpContext, api::ApiResponse, named_routes_axum, prelude::*},
    session_contract::Session,
};
use dirtybase_helper::hash::sha256;

use crate::{
    AuthConfig, guards::session_guard::auth_session::AuthSession, helpers::get_auth_storage,
};

pub(crate) async fn login_form_handler(
    RequestContext(context): RequestContext,
) -> impl IntoResponse {
    let mut submit_uri = named_routes_axum::helpers::get_path("auth:do-signin");
    if let Ok(auth_config) = context.get_config::<AuthConfig>("auth").await {
        submit_uri = named_routes_axum::helpers::get_path(&auth_config.auth_route());
    }

    Html(
            format!("<h1>Login Form</h1>
          <form method='post' action='{submit_uri}'>
        <label>Username: </label><input type='text' name='username' placeholder='username' value='admin' /> <br/>
        <label>Password: </label><input type='password' name='password' placeholder='password' value='password' /> <br/>
        <button type='submit'>Login</button>
        <p>
            <a href='/test'>Login With Google</a>
        </p>
        <p>
             <a href='/auth/register-form'>Register </a>
        </p>
      </form>"),
        )
}

pub(crate) async fn handle_login_request(
    RequestContext(ctx): RequestContext,
    CtxExt(http_ctx): CtxExt<HttpContext>,
    Form(cred): Form<LoginCredential>,
) -> Response<Body> {
    // TODO: This will use the auth service in the future
    let storage = if let Ok(s) = get_auth_storage(ctx.clone(), None).await {
        s
    } else {
        let bdy = Body::empty();
        return bdy.into_response();
    };

    // FIXME: handle error...
    let session = ctx.get::<Session>().await.unwrap();

    let result = if cred.username().is_some() {
        storage
            .find_by_username(cred.username().as_ref().unwrap())
            .await
    } else {
        let hash = sha256::hash_str(&cred.email().cloned().unwrap_or(":::nothing:::".to_string()));
        storage.find_by_email_hash(&hash).await
    };
    if let Ok(Some(user)) = result
        && user.verify_password(cred.password())
    {
        let auth_session = AuthSession::from_session(&session).await;
        http_ctx
            .set_cookie(AuthSession::new(user.id()).to_cookie(&session).await)
            .await;

        let mut response = ().into_response();

        response.headers_mut().append(
            header::LOCATION,
            header::HeaderValue::from_str(auth_session.redirect()).unwrap(),
        );
        *response.status_mut() = StatusCode::SEE_OTHER;
        return response;
    }

    let bdy = Body::empty();
    bdy.into_response()
}

pub(crate) async fn handle_logout_request(
    RequestContext(ctx): RequestContext,
) -> impl IntoResponse {
    let session = ctx.get::<Session>().await.unwrap();
    let auth_session = AuthSession::from_session(&session).await;
    _ = auth_session.delete(session, &ctx).await;

    let mut response = ().into_response();
    response.headers_mut().append(
        header::LOCATION,
        header::HeaderValue::from_str("/").unwrap(),
    );
    *response.status_mut() = StatusCode::FOUND;
    response
}

pub(crate) async fn handle_get_auth_token(
    RequestContext(ctx): RequestContext,
    Json(cred): Json<LoginCredential>,
) -> impl IntoResponse {
    // TODO: This will use the auth service in the future
    let storage = if let Ok(s) = get_auth_storage(ctx.clone(), None).await {
        s
    } else {
        return ApiResponse::<String>::error("could not resolve storage");
    };

    let result = if cred.username().is_some() {
        storage
            .find_by_username(cred.username().as_ref().unwrap())
            .await
    } else {
        let hash = sha256::hash_str(&cred.email().cloned().unwrap_or(":::nothing:::".to_string()));
        storage.find_by_email_hash(&hash).await
    };

    let mut res = ApiResponse::<String>::default();

    if let Ok(Some(user)) = result
        && user.verify_password(cred.password())
        && let Some(token) = user.generate_token()
    {
        res.set_data(token);
    }

    if !res.has_data() {
        res.set_error("authentication failed");
    }

    res
}

pub(crate) async fn handle_get_user_by_id(
    Path(id): Path<ArcUuid7>,
    CtxExt(storage): CtxExt<AuthUserStorageProvider>,
) -> ApiResponse<AuthUser> {
    storage.find_by_id(id).await.into()
}

pub(crate) async fn register_form_handler(
    CtxExt(http_context): CtxExt<HttpContext>,
) -> impl IntoResponse {
    let do_signup_route = http_context
        .named_route_service()
        .get("auth:do-signup-form")
        .unwrap();
    Html(
        format!(
        "<h1>Register Form</h1><form method='post' action='{}'>
    <label>Username: </label><input type='text' name='username' placeholder='username' /> <br/>
    <label>Email: </label><input type='text' name='email' placeholder='email' /> <br/>
    <label>Password: </label><input type='password' name='password' placeholder='password' /> <br/>
    <label>Confirm Password: </label><input type='password' name='confirm_password' placeholder='password' /> <br/>
    <button type='submit'>Register</button>
  </form>",
         do_signup_route.redirector().path()
        )
    )
}

pub(crate) async fn handle_register_request(
    RequestContext(ctx): RequestContext,
    Form(mut payload): Form<AuthUserPayload>,
) -> impl IntoResponse {
    // FIXME: This will use the auth service in the future
    let storage = if let Ok(s) = get_auth_storage(ctx.clone(), None).await {
        s
    } else {
        return "token:".to_string();
    };

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

    format!("token: {token}")
}

pub(crate) async fn handle_api_register_request(
    RequestContext(ctx): RequestContext,
    Json(mut payload): Json<AuthUserPayload>,
) -> ApiResponse<String> {
    // This will use the auth service in the future
    let storage = if let Ok(s) = get_auth_storage(ctx.clone(), None).await {
        s
    } else {
        let mut resp = ApiResponse::<String>::default();
        resp.set_error("could not register user");
        return resp;
    };

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
