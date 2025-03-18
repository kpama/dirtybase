mod migration;

use dirtybase_contract::{
    ExtensionMigrations, ExtensionSetup,
    app::{Context, RequestContext},
    auth::{AuthUserPayload, LoginCredential, StorageResolverPipeline},
    axum::{Form, Json, response::Html},
    http::{RouterManager, WebMiddlewareManager},
    prelude::IntoResponse,
};
use dirtybase_helper::hash::sha256;
use serde::{Deserialize, Serialize};

use crate::{
    AuthConfig, DATABASE_STORAGE, middlewares::setup_middlewares, register_storages,
    setup_context_managers,
};

#[derive(Debug, Default)]
pub struct Extension {
    is_enable: bool,
    is_db_storage: bool,
}

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, global_context: &Context) {
        let global_config = global_context
            .get_config::<AuthConfig>("dirtybase::auth")
            .await
            .unwrap();

        self.is_enable = global_config.is_enabled();
        self.is_db_storage = global_config.storage_ref().as_str() == DATABASE_STORAGE;

        if !self.is_enable {
            tracing::debug!("Auth is not enabled");
            return;
        }

        self.global_container().set_type(global_config).await;

        register_storages().await;
        setup_context_managers().await;
    }

    fn migrations(&self, _global_context: &Context) -> Option<ExtensionMigrations> {
        if !self.is_enable {
            return None;
        }

        if self.is_db_storage {
            return migration::setup();
        }

        None
    }

    fn register_web_middlewares(&self, manager: WebMiddlewareManager) -> WebMiddlewareManager {
        if !self.is_enable {
            return manager;
        }

        setup_middlewares(manager)
    }

    fn register_routes(
        &self,
        mut manager: RouterManager,
        _middleware_manager: &WebMiddlewareManager,
    ) -> RouterManager {
        manager.general(Some("/auth"), |router| {
            router
                .get("/login-form", login_form_handler, "auth::signin-form")
                .post("/do-login", || async { "do login" }, "auth::do-signin")
                .post("/my-token", handle_get_auth_token, "auth:get-token")
                .get(
                    "/register-form",
                    register_form_handler,
                    "auth:register-form",
                )
                .post(
                    "/do-registration",
                    handle_register_request,
                    "auth:do-register-form",
                )
        });
        manager
    }
}

async fn login_form_handler() -> impl IntoResponse {
    Html(
        "<h1>Login Form</h1><form method='post' action='/auth/do-registration'>
    <label>Username: </label><input type='text' name='username' placeholder='username' /> <br/>
    <label>Password: </label><input type='password' name='password' placeholder='password' /> <br/>
    <button type='submit'>Login</button>
    <p>
         <a href='/auth/register-form'>Register </a>
    </p>
  </form>",
    )
}

async fn handle_login_request(Form(cred): Form<LoginCredential>) -> impl IntoResponse {
    ""
}

async fn handle_get_auth_token(
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

    if let Ok(Some(user)) = result {
        if user.verify_password(cred.password()) {
            return Json(TokenResponse {
                success: true,
                token: user.generate_token(),
            });
        }
    }

    Json(TokenResponse {
        success: false,
        token: None,
    })
}

async fn register_form_handler() -> impl IntoResponse {
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

async fn handle_register_request(
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
struct RegisterData {
    username: String,
    email: String,
    password: String,
    confirm_password: String,
}

#[derive(Debug, Serialize)]
struct TokenResponse {
    success: bool,
    token: Option<String>,
}
