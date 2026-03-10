use axum::{Json, response::Html};
use dirtybase_contract::{
    auth_contract::{Actor, AuthUser, Gate},
    http_contract::Bind,
    prelude::{CtxExt, OptionCtxExt},
};
use dirtybase_db::types::ArcUuid7;
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");
    let app = dirtybase_app::setup().await.unwrap();

    Gate::define("has-access", || async {
        //
        Some(true)
    })
    .await;

    // Bind::<Actor>::alias("auth-actor").await;
    Bind::<Actor>::from_to::<ArcUuid7>("auth-actor", None).await;

    app.setup_web(|mut manager, _middleware_manager| {
        manager.general(None, |router| {
            router.get_x("/", || async { Html("Home page") });
            router.get_x("/u/{auth-actor}", |CtxExt(actor): CtxExt<Actor>| async  move { 
                //
                Json(actor)
             });
            router.get_x_with_middleware(
                "/secure",
                |CtxExt(user): CtxExt<AuthUser>, OptionCtxExt(actor_opt): OptionCtxExt<Actor>| async move {
                    tracing::warn!("auth user: {:#?}", user.username());
                    if let Some(actor) =  actor_opt {
                        if  actor.can("secrets:view") {
                            return Html("One ring to rule them all is the big secret");
                        }
                    } else {
                        return Html("you don't have permission");
                    }


                    Html("This is half of the secret. You don't have permission to view all")
                },
                ["auth", "can:has-access"],
            );
            router.group_with_middleware(
                "/foo",
                |r| {
                    r.get_x("/", || async {
                        Html("Hello world from middleware example")
                    });
                },
                ["auth::session"],
            );
        });
        manager
    })
    .await;

    _ = dirtybase_app::run_http(app).await;
}
