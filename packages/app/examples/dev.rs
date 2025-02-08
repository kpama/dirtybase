use std::collections::HashMap;

use axum_extra::extract::{CookieJar, Host};
use dirtybase_app::{run, setup};
use dirtybase_auth::middlewares::{handle_user_login_web_request, UserCredential};
use dirtybase_contract::config::DirtyConfig;
use dirtybase_contract::{
    app::{Context, ContextManager, CtxExt},
    prelude::*,
    session::Session,
};
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .try_init()
        .expect("could not setup tracing");

    let app_service = setup().await.unwrap();

    app_service.register(App).await;

    // _ = dirtybase_app::run_command(["serve"]).await;
    _ = run(app_service).await;
}

struct App;

#[async_trait::async_trait]
impl ExtensionSetup for App {
    async fn setup(&mut self, _config: &DirtyConfig) {
        busybody::helpers::register_service(UserProviderService::new(MyOwnUserProvider));
        busybody::helpers::register_service(ContextManager::<i32>::new());
    }

    fn register_cli_middlewares(&self, mut manager: CliMiddlewareManager) -> CliMiddlewareManager {
        manager.register("say_hi", |middleware| {
            middleware.next(|v, n| {
                Box::pin(async {
                    println!("I am saying hi from say_hi middleware");
                    n.call(v).await
                })
            });

            middleware
        });
        manager
    }

    fn register_routes(
        &self,
        mut manager: RouterManager,
        middleware_manager: &WebMiddlewareManager,
    ) -> RouterManager {
        manager.general(None, |router| {
            let router = router.get("/", index_request_handler, "index-page");
            // middleware_manager.apply(router, ["auth::normal"])
            router
        });

        // login
        manager.general(None, |router| {
            router
                .post(
                    "/do-login",
                    |Form(credential): Form<UserCredential>| async move {
                        println!("credential: {:#?}", credential);
                        "Auth finish"
                    },
                    "do-login",
                )
                .post("/do-login2", handle_user_login_web_request, "do-login2") // FIXME: CSRF Token feature...
                .get("/xx", test_cookie_handler, "xx")
        });

        manager
    }
}

async fn test_cookie_handler(
    jar: CookieJar,
    CtxExt(session): CtxExt<Session>,
    _req: Request,
) -> impl IntoResponse {
    session.id().to_string()
}

async fn index_request_handler(
    CtxExt(session): CtxExt<Session>,
    context: Extension<Context>,
    Host(hostname): Host,
    req: Request,
) -> impl IntoResponse {
    context.metadata().add("index handler", true.to_string());

    log::info!("in index page");
    if let Some(user) = context.user() {
        println!("current user: {:?}", user);
        println!("current user is the global user? {}", user.is_global());
        let uri = Uri::builder()
            .scheme("https")
            .authority("username:password@yahoo.com")
            .path_and_query("/foo/bar?one=1&two=1")
            .build()
            .unwrap();
        tracing::error!("generated uri: {}", uri);
        tracing::error!("request uri: {:?}", req.uri().scheme_str());

        format!(
            "Welcome to our secure application. user id {}. host: {}",
            user.id(),
            hostname
        )
    } else {
        "Welcome unknown user".to_string()
    }
}

struct MyOwnUserProvider;

#[async_trait::async_trait]
impl UserProviderTrait for MyOwnUserProvider {
    async fn by_username(&self, id: &str) -> String {
        println!("using a custom user serivce provider");
        let mut dev_users = HashMap::new();

        dev_users.insert("user1", "pwd1");
        dev_users.insert("user2", "pwd2");
        dev_users.insert("user3", "pwd3");

        if let Some(st) = dev_users.get(id) {
            st.to_string()
        } else {
            String::new()
        }
    }

    async fn by_email(&self, _email: &str) -> String {
        String::new()
    }

    async fn by_id(&self, _id: &str) -> String {
        String::new()
    }
}
