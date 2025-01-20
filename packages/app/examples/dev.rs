use std::collections::HashMap;

use axum_extra::extract::{cookie::Cookie, CookieJar};
use dirtybase_app::{run, setup};
use dirtybase_auth::middlewares::{handle_user_login_request, UserCredential};
use dirtybase_config::DirtyConfig;
use dirtybase_contract::{
    app::{Context, ContextManager},
    prelude::*,
};

#[tokio::main]
async fn main() {
    let app_service = setup().await.unwrap();

    app_service.register(App).await;

    // _ = dirtybase_app::run_command(["serve"]).await;
    _ = run(app_service).await;
}

struct App;

#[async_trait::async_trait]
impl ExtensionSetup for App {
    async fn setup(&self, _config: &DirtyConfig) {
        busybody::helpers::register_service(UserProviderService::new(MyOwnUserProvider));
        busybody::helpers::register_service(ContextManager::<i32>::new());
    }

    fn register_routes(
        &self,
        mut manager: RouterManager,
        middleware_manager: &WebMiddlewareManager,
    ) -> RouterManager {
        manager.general(None, |router| {
            let router = router.get_x("/", index_request_handler);
            // .middleware(|req, next| async {
            //     let context = req.extensions().get::<Context>().unwrap();
            //     let user = context.user().unwrap();
            //     println!(">>>>>>>>>>> last middleware to run: {:?}", user.role());
            //     next.run(req).await
            // });

            middleware_manager.apply(router, ["auth:jwt", "auth:basic", "auth:normal"])
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
                .post("/do-login2", handle_user_login_request, "do-login2") // FIXME: CSRF Token feature...
                .get("/xx", test_cookie_handler, "xx")
        });

        manager
    }
}

async fn test_cookie_handler(mut jar: CookieJar, req: Request) -> impl IntoResponse {
    println!("testing setting cookies");
    let entry = Cookie::new("age", 2.to_string());
    jar = jar.add(entry);
    (jar, "We are ready to rumble")
}

async fn index_request_handler(req: Request) -> impl IntoResponse {
    println!("handling the request");
    let ctx = req.extensions().get::<Context>().unwrap();
    let sc = ctx.service_container();
    let user = ctx.user().unwrap();
    println!("is container a proxy? {}", sc.is_proxy());
    println!("current user: {:?}", user);

    // if let Some(manager) = ctx.service_container().get::<ContextManager<i32>>() {
    //     let count = manager
    //         .context("tenent1", 5, || {
    //             println!(">>>>>>>>>>>>>>>>>>>> getting the counter....");
    //             Box::pin(async {
    //                 tokio::time::sleep(Duration::from_secs(10)).await;
    //                 1000
    //             })
    //         })
    //         .await;

    //     println!("current count: {}", count);
    // }

    // let req_sc = req.extensions().get::<ServiceContainer>();
    // println!("we got the user id via request: {}", req_sc.is_some());

    format!(
        "Welcome to our secure application. user context id: {}",
        user.id()
    )
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
