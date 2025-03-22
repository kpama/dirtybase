use axum::response::Html;
use axum_extra::extract::CookieJar;
use dirtybase_app::{run, setup};
use dirtybase_contract::app::RequestContext;
use dirtybase_contract::cli::CliMiddlewareManager;
use dirtybase_contract::{
    app::{Context, ContextResourceManager, CtxExt},
    prelude::*,
    session::Session,
};
use dirtybase_db::base::manager::Manager;
use dirtybase_db::types::{ArcUuid7, IntoColumnAndValue};
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::ERROR)
        .try_init()
        .expect("could not setup tracing");

    let app_service = setup().await.unwrap();

    app_service.register(App).await;

    _ = run(app_service).await;
}

#[derive(Default)]
struct App;

#[async_trait::async_trait]
impl ExtensionSetup for App {
    async fn setup(&mut self, _context: &Context) {
        busybody::helpers::register_service(ContextResourceManager::<i32>::new(
            |_| Box::pin(async { ("global points".to_string(), 50) }),
            |_| {
                Box::pin(async {
                    tracing::error!(">>>>>>>>>>>>>>>>>>>>>>>  making new i32");
                    40000
                })
            },
            |_| Box::pin(async {}),
        ))
        .await;
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

    fn register_routes(&self, manager: &mut RouterManager) {
        manager.general(None, |router| {
            let router = router.get("/", index_request_handler, "index-page");
            // middleware_manager.apply(router, ["auth::jwt"])
        });

        manager.api(None, |router| {
            router.get_x("/hello", || async { "Hello from api" });
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
                        _ = manager.insert_ref("auth_users", &auth_user).await;
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
                );
        });
    }

    async fn on_web_request(&self, req: Request, context: Context, _cookie: &CookieJar) -> Request {
        let tenant = context.tenant().await.unwrap();

        let id = tenant.id().to_string();
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

async fn index_request_handler() -> impl IntoResponse {
    "Index page"
}
