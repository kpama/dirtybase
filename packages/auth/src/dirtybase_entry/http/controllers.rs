use dirtybase_contract::{
    app::RequestContext,
    auth::{AuthUserPayload, LoginCredential, StorageResolverPipeline},
    axum::response::Html,
    http::{api::ApiResponse, prelude::*},
};
use dirtybase_helper::hash::sha256;
use serde::{Deserialize, Serialize};

pub(crate) async fn login_form_handler() -> impl IntoResponse {
    Html(
        "<h1>Login Form</h1>
      <form method='post' action='/auth/do-registration'>
    <label>Username: </label><input type='text' name='username' placeholder='username' /> <br/>
    <label>Password: </label><input type='password' name='password' placeholder='password' /> <br/>
    <button type='submit'>Login</button>
    <p>
         <a href='/auth/register-form'>Register </a>
    </p>
  </form>",
    )
}

pub(crate) async fn handle_login_request(Form(cred): Form<LoginCredential>) -> impl IntoResponse {
    ""
}

pub(crate) async fn handle_get_auth_token(
    RequestContext(ctx): RequestContext,
    Json(cred): Json<LoginCredential>,
) -> impl IntoResponse {
    // This will use the auth service in the future
    let storage = StorageResolverPipeline::new(ctx)
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
    Form(data): Form<RegisterData>,
) -> impl IntoResponse {
    // This will use the auth service in the future
    let storage = StorageResolverPipeline::new(ctx)
        .get_provider()
        .await
        .unwrap();
    let mut payload = AuthUserPayload::default();
    payload.username = Some(data.username.clone());
    payload.email = Some(data.email.clone());
    payload.password = Some(data.password.clone());
    payload.rotate_salt = true;
    payload.verified_at = Some(dirtybase_helper::time::current_datetime());

    if let Ok(user) = storage.store(payload).await {
        format!("token: {}", user.generate_token().unwrap())
    } else {
        format!("token: ")
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterData {
    username: String,
    email: String,
    password: String,
    confirm_password: String,
}
