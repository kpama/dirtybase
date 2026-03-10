use axum::response::Html;
use dirtybase_app::core::AppService;
use dirtybase_contract::{
    ExtensionSetup,
    auth_contract::{Actor, AuthUser, Gate},
    prelude::{
        Context, CtxExt, Observable, RouterManager,
        observable::{CelContext, CommonExpressionSandbox},
    },
};
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        // .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");

    MyApp.register().await;

    dirtybase_app::setup_and_run()
        .await
        .expect("could not setup and run the app");
}

struct MyApp;

#[async_trait::async_trait]
impl ExtensionSetup for MyApp {
    async fn setup(&mut self, _ctx: &Context) {
        CelContext::subscribe(|mut common, app_ctx| async move {
            let app = app_ctx
                .get::<AppService>()
                .await
                .expect("could not get the app service");
            let app_config = app.config_ref();
            common
                .add_variable("env", app_config.environment())
                .expect("could not add env variable");

            common
        })
        .await;

        CommonExpressionSandbox::subscribe(|manager, _| async move {
            tracing::error!("registering is_prod programming");
            manager.set_program("is_prod", "env == 'prod'");
            manager
        })
        .await;

        Gate::define(
            "profile:edit",
            |context: Context, actor: Actor| async move {
                if let Ok(manager) = context.get::<CommonExpressionSandbox>().await {
                    tracing::error!(
                        "in production ? {:?}",
                        manager.execute(&context, "is_prod").await
                    );
                }
                tracing::info!("actor can profile:edit: {}", actor.can("profile:edit"));
                Some(actor.can("profile:edit"))
            },
        )
        .await;
    }

    /// Register HTTP routes
    fn register_routes(&self, manager: &mut RouterManager) {
        manager.general(None, |router| {
            // home page
            router.get_x("/", |CtxExt(ctx): CtxExt<Context>| async move {
                let manager = CommonExpressionSandbox::new().await;
                match manager.execute(&ctx, "is_prod").await {
                    Ok(ans) => {
                        if !ans.is_zero() {
                            Html("<h1>Welcome to prod</h1>")
                        } else {
                            Html("<h1>Welcome to dev</h1>")
                        }
                    }
                    Err(e) => {
                        tracing::error!("cel execution error: {:#?}", e);
                        Html("Could not deduced the current environment")
                    }
                }
            });

            // my info
            router.get_x_with_middleware(
                "/my-profile",
                |CtxExt(gate): CtxExt<Gate>, CtxExt(auth_user): CtxExt<AuthUser>| async move {
                    //
                    if gate.can("profile:edit").await {
                        Html(format!(
                            "
                      <h1>Edit Profile</h1>
                      <form>
                         <label for=\"name\">Username</label>
                         <input type=\"text\" id=\"name\" value=\"{}\" />
                      </form>
                      ",
                            auth_user.username()
                        ))
                    } else {
                        Html("<h1>User profile page</h1>".to_string())
                    }
                },
                ["auth"],
            );
        });
    }
}
