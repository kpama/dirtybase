use std::{collections::HashMap, sync::Arc};

use axum::{Json, extract::Path, response::Html};
use dirtybase_contract::{
    ExtensionSetup,
    auth_contract::Gate,
    http_contract::HttpContext,
    prelude::{Bearer, Context, Credentials, CtxExt, RouterManager},
};
use serde::Serialize;
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(Level::TRACE)
        .try_init()
        .expect("could not setup tracing");
    let app = dirtybase_app::setup().await.unwrap();

    //
    // register the our application
    app.register(OurApp).await;

    _ = dirtybase_app::run(app).await;
}

type AppStore = Arc<HashMap<i32, i32>>;

struct OurApp;

#[async_trait::async_trait]
impl ExtensionSetup for OurApp {
    async fn setup(&mut self, global_context: &Context) {
        let mut discounts = HashMap::<i32, i32>::new();
        discounts.insert(1, 20);
        discounts.insert(2, 20);
        discounts.insert(3, 25);
        discounts.insert(10, 50);

        global_context.set(AppStore::new(discounts)).await;

        Gate::define("view_discount", |ctx: Context| async move {
            //
            if let Ok(http_ctx) = ctx.get::<HttpContext>().await {
                let Some(auth_header) = http_ctx.header("Authorization") else {
                    return false.into();
                };
                let Some(token) = Bearer::decode(&auth_header) else {
                    return false.into();
                };

                if token.token() != "1234" {
                    return false.into();
                }
            } else {
                return false.into();
            }

            true.into()
        })
        .await;
    }

    /// Register HTTP routes
    fn register_routes(&self, manager: &mut RouterManager) {
        manager.general(None, |router| {
            router.get_x("/", || async {
                Html("<h1>Welcome to our application</h1>")
            });

            router.get_x_with_middleware(
                "/discount/{id}",
                show_discount_of_day,
                ["can:view_discount"],
            );
        });
    }
}

async fn show_discount_of_day(
    Path(id): Path<i32>,
    CtxExt(store): CtxExt<AppStore>,
) -> Json<Discount> {
    // -
    let discount = store.get(&id).cloned().unwrap_or_default();

    Json(Discount {
        percentage: discount,
    })
}

#[derive(Debug, Default, Serialize)]
struct Discount {
    percentage: i32,
}
