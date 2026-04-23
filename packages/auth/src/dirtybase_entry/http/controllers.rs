use dirtybase_contract::{
    app_contract::{CtxExt, RequestContext},
    auth_contract::{
        Actor, ActorPayload, FetchActorPayload, LoginCredential, PersistActorPayload,
        storage::{PermStorageProvider, PermissionStorage},
    },
    axum::response::Html,
    db_contract::types::ArcUuid7,
    http_contract::{HttpContext, api::ApiResponse, named_routes_axum, prelude::*},
    session_contract::Session,
};
use jwt::ToBase64;

use crate::{
    AuthConfig, AuthExtension, guards::session_guard::auth_session::AuthSession,
    helpers::get_auth_storage,
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
    let storage = if let Ok(s) = get_auth_storage(ctx.clone()).await {
        s
    } else {
        let bdy = Body::empty();
        return bdy.into_response();
    };

    // FIXME: handle error...
    let session = ctx.get::<Session>().await.unwrap();

    let result = if cred.username().is_some() {
        let payload = FetchActorPayload::by_username(cred.username().as_ref().cloned().unwrap());
        storage.fetch_actor(payload, None).await
    } else {
        let payload = FetchActorPayload::by_email(&cred.email().cloned().unwrap_or("".to_string()));
        storage.fetch_actor(payload, None).await
    };
    if let Ok(Some(user)) = result
        && user.verify_password(cred.password())
    {
        let header = if let Some(auth_session) = AuthSession::from_session(&session).await {
            HeaderValue::from_str(auth_session.redirect().as_ref())
                .expect("Could not create header from auth session")
        } else {
            HeaderValue::from_str("/").unwrap() // NOTE: Should never panic as we set the value here
        };

        http_ctx
            .set_cookie(
                AuthSession::new(user.id().cloned())
                    .to_cookie(&session)
                    .await,
            )
            .await;

        let mut response = ().into_response();
        response.headers_mut().append(header::LOCATION, header);
        *response.status_mut() = StatusCode::SEE_OTHER;
        return response;
    }

    // TODO: Notify observers regarding the redirect back to the login form
    if let Ok(auth_config) = AuthExtension::config_from_ctx(&ctx).await {
        let login_form = auth_config.signin_form_route();
        named_routes_axum::helpers::redirect(&login_form).into_response()
    } else {
        let mut response = ().into_response();
        let header = HeaderValue::from_str("/").unwrap(); // NOTE: Unwrap is okay here..
        response.headers_mut().append(header::LOCATION, header);
        response
    }
}

pub(crate) async fn handle_logout_request(
    RequestContext(ctx): RequestContext,
) -> impl IntoResponse {
    let session = ctx.get::<Session>().await.unwrap();
    if let Some(auth_session) = AuthSession::from_session(&session).await {
        _ = auth_session.delete(session, &ctx).await;
    }

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
    let storage = if let Ok(s) = get_auth_storage(ctx.clone()).await {
        s
    } else {
        return ApiResponse::<String>::error("could not resolve storage");
    };

    let result = if cred.username().is_some() {
        let payload = FetchActorPayload::by_username(cred.username().as_ref().unwrap());
        storage.fetch_actor(payload, None).await
    } else {
        let payload = FetchActorPayload::by_email(&cred.email().cloned().unwrap_or("".to_string()));
        storage.fetch_actor(payload, None).await
    };

    let mut res = ApiResponse::<String>::default();

    if let Ok(Some(user)) = result
        && user.verify_password(cred.password())
        && let Some(token) = user.generate_token()
    {
        let key = b"the quick brown fox jumps over";
        let claim = user
            .generate_signed_jwt(key)
            .expect("could not generate jwt");
        dbg!("{}", &claim);
        res.set_data(claim);
    }

    if !res.has_data() {
        res.set_error("authentication failed");
    }

    res
}

pub(crate) async fn handle_get_user_by_id(
    Path(id): Path<ArcUuid7>,
    CtxExt(storage): CtxExt<PermStorageProvider>,
) -> ApiResponse<Actor> {
    let payload = FetchActorPayload::by_id(id);
    storage.fetch_actor(payload, None).await.into()
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
    Form(mut payload): Form<ActorPayload>,
) -> impl IntoResponse {
    // FIXME: This will use the auth service in the future
    let storage = if let Ok(s) = get_auth_storage(ctx.clone()).await {
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
    let payload = PersistActorPayload::Save {
        actor: payload.into(),
    };
    if let Ok(Some(actor)) = storage.save_actor(payload).await {
        match actor.generate_token() {
            Some(t) => token = t,
            None => {
                tracing::error!("did not get back user token: {:?}", actor.id())
            }
        }
    }

    format!("token: {token}")
}

pub(crate) async fn handle_api_register_request(
    RequestContext(ctx): RequestContext,
    Json(mut payload): Json<ActorPayload>,
) -> ApiResponse<String> {
    // This will use the auth service in the future
    let storage = if let Ok(s) = get_auth_storage(ctx.clone()).await {
        s
    } else {
        let mut resp = ApiResponse::<String>::default();
        resp.set_error("could not register user");
        return resp;
    };

    payload.rotate_salt = true;
    let mut resp = ApiResponse::<String>::default();

    let payload = PersistActorPayload::Save {
        actor: payload.into(),
    };
    if let Ok(Some(actor)) = storage.save_actor(payload).await {
        resp.set_data(actor.generate_token().unwrap());
    } else {
        resp.set_error("could not register user");
    }
    resp
}

pub(crate) async fn handle_api_get_me(CtxExt(actor): CtxExt<Actor>) -> ApiResponse<Actor> {
    ApiResponse::success(actor)
}
