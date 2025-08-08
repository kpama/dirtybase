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
use dirtybase_auth::{guards::session_guard, helpers::get_auth_storage};
use dirtybase_contract::{
    ExtensionSetup,
    prelude::{ConfigResult, Context, CtxExt, DirtyConfig, RouterManager, TryFromDirtyConfig},
};
use dirtybase_db::{ types::ArcUuid7};
use serde::Deserialize;
use tracing::Level;

// G_CLIENT_ID
// G_SECRET

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::ERROR)
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

            router.get("/test", |CtxExt(config): CtxExt<OauthConfig>| async move {
                let client_id= config.client_id;
                let nonce = ArcUuid7::default().to_uuid25_string();
                let state = ArcUuid7::default().to_uuid25_string();
                let params = &[
                        ("response_type", "code"),
                        ("client_id", &client_id),
                        ("scope", "openid profile email"),
                        ("redirect_uri", "https://rustdev.mansaray.me/google-auth"),
                        ("state", &state),
                        ("nonce", &nonce)
                        ];
            //     let url =  format!("https://accounts.google.com/o/oauth2/v2/auth?{}&{}", serde_urlencoded::to_string(params).unwrap(),
            //     "scope=openid%20profile%20email"
            // );

                let url =  format!("https://accounts.google.com/o/oauth2/v2/auth?{}", serde_urlencoded::to_string(params).unwrap());

                Html(format!("<a href='{}'>Login with google</a>",  url))

                //url
                //  Redirect::temporary(&url)
            },  "oauth-login");

            router.get_x_with_middleware(
                "/secure",
                || async{
                        Html("<h1>Welcome to the secure page<h1>")
                },
                ["auth:oauth"]
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
                            let data = 
                        response.text().await.unwrap();
                    // _ = fs::write(
                    //     "user_info.json",
                    //     &data.as_bytes()
                    // );
                        // check if the user already has data stored

                        let auth_prov = get_auth_storage(ctx.clone(), None).await.unwrap();
                        if let Ok(Some(user))= auth_prov.find_by_username("foouser").await {
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
