use axum::response::Html;
use axum_extra::extract::CookieJar;
use dirtybase_app::{run, setup};
use dirtybase_contract::app::RequestContext;
use dirtybase_contract::config::DirtyConfig;
use dirtybase_contract::{
    app::{Context, ContextManager, CtxExt},
    prelude::*,
    session::Session,
};
use dirtybase_db::base::manager::Manager;
use dirtybase_db::types::{ArcUuid7, IntoColumnAndValue};
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
            middleware_manager.apply(router, ["auth::normal"])
        });

        // login
        manager.general(None, |router| {
            router
                .get_x("/form", || async move {
                    let form = r#"<form action='/do-login', method='post'>
                    <label>Username</label><br/>
                    <input type='text' name='username' placeholer='Username' /> <br/>
                    <label>Password</label><br/>
                    <input type='password' name='password' placeholer='Pasword' /> <br/>
                    <input type='submit' value='Login'/>
                    </form>"#;
                    Html(form)
                })
                .post(
                    "/do-login", // TODO: CSRF CHECK
                    |credential: LoginCredential| async move {
                        println!("credential - username : {:#?}", credential.username());
                        println!("credential - password: {:#?}", credential.password());
                        "Auth finish"
                    },
                    "do-login",
                )
                .get("/xx", test_cookie_handler, "xx")
                .post_x(
                    "/create-user",
                    |CtxExt(manager): CtxExt<Manager>,
                     Form(mut auth_user): Form<AuthUserPayload>| async move {
                        let result = auth_user.validate();
                        auth_user.id = Some(ArcUuid7::default());
                        manager.insert_ref("auth_users", &auth_user).await;
                        tracing::error!("validation result: {:#?}", result);
                        format!("new user id: {}", auth_user.id.unwrap())
                    },
                )
                .put_x(
                    "/update-user/{id}",
                    |Path(id): Path<ArcUuid7>, Form(mut data): Form<AuthUserPayload>| async move {
                        data.id = Some(id.clone());
                        tracing::error!("updating{:#?}", &data);
                        tracing::error!("column/value: {:#?}", data.into_column_value());
                        format!("updated user id: {}", &id)
                    },
                )
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
