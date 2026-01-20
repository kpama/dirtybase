use axum::response::Html;
use axum_extra::extract::CookieJar;
use dirtybase_app::{run, setup};
use dirtybase_contract::auth_contract::{AuthUserPayload, LoginCredential};
use dirtybase_contract::cli_contract::CliMiddlewareManager;
use dirtybase_contract::multitenant_contract::model::{Tenant, TenantStatus};
use dirtybase_contract::multitenant_contract::{TenantManager, TenantStorage};
use dirtybase_contract::{
    app_contract::{Context, ContextResourceManager, CtxExt},
    prelude::*,
    session_contract::Session,
};
use dirtybase_db::base::manager::Manager;
use dirtybase_db::types::{ArcUuid7, ToColumnAndValue};
use tracing_subscriber::EnvFilter;
use validator::Validate;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        // .with_max_level(Level::DEBUG)
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
            |_| {
                Box::pin(async {
                    //
                    Ok(("global points".to_string(), 50).into())
                })
            },
            |_| {
                Box::pin(async {
                    tracing::error!(">>>>>>>>>>>>>>>>>>>>>>>  making new i32");
                    Ok(40000)
                })
            },
            |_| Box::pin(async {}),
        ))
        .await;
    }

    fn register_cli_middlewares(&self, manager: CliMiddlewareManager) -> CliMiddlewareManager {
        manager
    }

    fn register_routes(&self, manager: &mut RouterManager) {
        manager.general(None, |router| {
            router.get("/", index_request_handler, "index-page");
            // manager.apply(router, ["auth::jwt"])
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
                    <input type='text' name='username' placeholder='Username' /> <br/>
                    <label>Password</label><br/>
                    <input type='password' name='password' placeholder='Password' /> <br/>
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
                        tracing::error!("column/value: {:#?}", data.to_column_value());
                        format!("updated user id: {}", &id)
                    },
                );
        });
    }

    async fn on_web_request(&self, req: Request, context: Context, _cookie: &CookieJar) -> Request {
        let tenant = context.tenant_context().await.unwrap();

        let _id = tenant.id().to_string();
        req
    }
}

async fn test_cookie_handler(
    _jar: CookieJar,
    CtxExt(session): CtxExt<Session>,
    _req: Request,
) -> impl IntoResponse {
    session.id().to_string()
}

async fn index_request_handler() -> impl IntoResponse {
    Html("<h1>Index page</h1>")
}
