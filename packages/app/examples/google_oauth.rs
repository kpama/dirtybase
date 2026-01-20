use std::{
    fmt::{Debug, Display},
    fs,
};

use axum::{
    Extension,
    extract::Query,
    response::{Html, Redirect},
};
use dirtybase_app::{run, setup};
use dirtybase_auth::{
    guards::{openid_guard::openid_client::OpenIdClient, session_guard},
    helpers::get_auth_storage,
};
use dirtybase_contract::{
    ExtensionSetup,
    http_contract::HttpContext,
    prelude::{ConfigResult, Context, CtxExt, DirtyConfig, RouterManager, TryFromDirtyConfig},
    session_contract::Session,
};
use serde::Deserialize;
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");

    tracing::debug!("starting up....");
    let app_service = setup().await.unwrap();

    _ = app_service.register(OauthApp).await;
    _ = app_service.init().await;

    _ = run(app_service).await;
}

struct OauthApp;

#[async_trait::async_trait]
impl ExtensionSetup for OauthApp {
    async fn setup(&mut self, global_context: &Context) {
        if let Ok(config) = global_context.get_config::<OauthConfig>("g_token").await {
            global_context.set(config).await;
        }
    }

    fn register_routes(&self, manager: &mut RouterManager) {
        manager.general(None, |router| {
            router.any_x("/", || async {
                Html("<h1>Hello world.</h1> You need to login for awesomeness <br/> <a href='/test'>Login</a>")
            });

            router.get("/test", |CtxExt(config): CtxExt<OauthConfig>, CtxExt(session): CtxExt<Session>| async move {

                let client = OpenIdClient::new(&config.client_id, &config.secret);
                let url_info =  client.build_redirect("https://accounts.google.com/o/oauth2/v2/auth", "https://rustdev.mansaray.me/google-auth", &["email"]);

                session.put("_openid", &url_info).await;

                Html(format!("<a href='{}'>Login with google</a>",  url_info.url()))

            },  "oauth-login");

            router.get_x_with_middleware(
                "/secure",
                |CtxExt(http_ctx): CtxExt<HttpContext>| async move {
                    let body = if let Some(route) = http_ctx.named_route_service().get("auth:logout") {
                        format!("<h1>Welcome to the secure page<h1><a href='{}'>Logout</a>", route.redirector().path())
                    } else {
                        "<h1>Welcome to the secure page<h1>".to_string()
                    };
                    Html(body)
                },
                ["auth:openid"]
            );

            router.get_x(
                "/google-auth",
                |Query(auth_code): Query<AuthCode>,
                Extension(ctx): Extension<Context>,
                CtxExt(config): CtxExt<OauthConfig>
                | async move {
                    log::error!("in google-auth");
                    let client = reqwest::Client::new();

                    if let Ok(response) =  client.post("https://oauth2.googleapis.com/token")  
                        .form(&[
                            ("code", auth_code.code.as_str()),
                            ("client_id", config.client_id.as_str()),
                            ("client_secret", config.secret.as_str()),
                            ("redirect_uri",  "https://rustdev.mansaray.me/google-auth"),
                            ("grant_type", "authorization_code")
                        ]).send().await {
                        // check if the user already has data stored
                        //
                        _= fs::write("token.json", response.text().await.unwrap());

                        let auth_prov = get_auth_storage(ctx.clone(), None).await.unwrap();
                        if let Ok(Some(user))= auth_prov.find_by_username("admin").await {
                            session_guard::log_user_in(user, ctx).await;
                        }

                        return Redirect::temporary("/secure");                     
                    }
                    Redirect::temporary("/")
                },
            );
        });
    }
}

#[derive(Debug, Deserialize)]
struct AuthCode {
    code: String,
}

#[derive(Deserialize, Clone)]
struct OauthConfig {
    client_id: String,
    secret: String,
}

#[async_trait::async_trait]
impl TryFromDirtyConfig for OauthConfig {
    type Returns = Self;

    async fn from_config(base: &DirtyConfig, _ctx: &Context) -> ConfigResult<Self::Returns> {
        let config = base
            .optional_file("g_token.toml", Some("G"))
            .build()
            .await?
            .try_deserialize::<Self>()?;

        Ok(config)
    }
}

impl Debug for OauthConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for OauthConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.client_id)
    }
}
