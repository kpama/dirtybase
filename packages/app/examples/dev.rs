use std::collections::HashMap;

use axum_extra::extract::CookieJar;
use dirtybase_app::{run, setup};
use dirtybase_auth::middlewares::{handle_user_login_web_request, UserCredential};
use dirtybase_contract::app::RequestContext;
use dirtybase_contract::config::DirtyConfig;
use dirtybase_contract::{
    app::{Context, ContextManager, CtxExt},
    prelude::*,
    session::Session,
};
use dirtybase_db::base::manager::Manager;
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

#[derive(Default)]
struct App;

#[async_trait::async_trait]
impl ExtensionSetup for App {
    async fn setup(&mut self, _config: &DirtyConfig) {
        busybody::helpers::register_service(UserProviderService::new(MyOwnUserProvider)).await;
        busybody::helpers::register_service(ContextManager::<i32>::new()).await;
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

    async fn on_web_request(&self, req: Request, context: Context, _cookie: &CookieJar) -> Request {
        let tenant = context.tenant().await.unwrap();

        let id = tenant.id().to_string();
        context
            .container()
            .resolver(move |c| {
                let id2 = id.clone();
                Box::pin(async move {
                    if let Some(m) = c.get::<ContextManager<i32>>().await {
                        println!(">>>>>>>>>>>>>>>>>>>> tenant id is <<<<< : {:?}", &id2);
                        // println!("still has context: {}", m.has_context(&id).await);
                        return m
                            .context(
                                &id2,
                                30,
                                || {
                                    Box::pin(async {
                                        tracing::error!(">>>>>>>>>>>>>>>>>>>>>>>  making new i32");
                                        40000
                                    })
                                },
                                |_| Box::pin(async {}),
                            )
                            .await;
                    }
                    3000
                })
            })
            .await;
        req
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
    CtxExt(number): CtxExt<i32>,
    CtxExt(manager): CtxExt<Manager>,
    RequestContext(context): RequestContext,
) -> impl IntoResponse {
    context
        .metadata()
        .await
        .add("index handler", true.to_string());
    let has_company = manager.has_table("companies").await;

    log::info!("in index page");
    if let Some(user) = context.user().await {
        println!("current user: {:?}", user);
        println!("current user is the global user? {}", user.is_global());

        format!(
            "Welcome to our secure application. user id {}. i32: {}. has companies: {}",
            user.id(),
            number,
            has_company
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
