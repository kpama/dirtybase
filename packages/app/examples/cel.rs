
use axum::{Json, extract::Path, response::Html};
use dirtybase_app::app::AppService;
use dirtybase_contract::{
    ExtensionSetup,
    auth_contract::{Actor, Gate},
    prelude::{
        Context, CtxExt, Observable, RouterManager,
        observable::cel::{CelContext, CommonExpressionSandbox},
    },
};
use dirtybase_db::{base::manager::Manager, types::{JsonField}};
use dirtybase_db_macro::DirtyTable;
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
                .add_variable("is_owner", true)
                .expect("could not add 'is_owner' variable");

            common
        })
        .await;

        CommonExpressionSandbox::subscribe(|manager, _| async move {
            tracing::error!("registering is_prod programming");
            manager.set_program("is_prod", "env == 'prod'");
            manager.set_program("user_is_allow", "_actor.id == '019cd845-5fca-7b80-9f3c-5068476a9df4'");
            manager.set_program("_actor", "_actor.id");
            manager
        })
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
                            let result =  manager.execute(&ctx, "user_is_allow").await;
                            if let Ok(ans) = result && ans.is_zero() {
                                Html("<h1>Welcome to dev. You do not have full access</h1>")
                            } else {
                                Html("<h1>Welcome to dev. You have full access</h1>")
                            }
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
                |CtxExt(gate): CtxExt<Gate>, CtxExt(actor): CtxExt<Actor>| async move {
                    //
                    if gate.can("profile:edit").await {
                        Html(format!(
                            "
                      <h1>Edit Profile, current login with role {}</h1>
                      <form>
                         <label for=\"name\">Username</label>
                         <input type=\"text\" id=\"name\" value=\"{}\" />
                         <p>ID: {:?} </p>
                      </form>
                      ",
                            actor.current_role().name(),
                            actor.username_ref(),
                            actor.id()
                        ))
                     } else if gate.can("posts:view").await {
                       Html(format!("<h1>This is your personal post</>, The quick brown fox jumps over the lazy dog. <p>by {} <br/>, Id:{:?}</p>", actor.username_ref(), actor.id()))
                    } else {
                        Html(format!(
                            "<h1>User {}'s profile page. login with role {}, id: {:?}</h1>",
                            actor.username_ref(),
                            actor.current_role().name(),
                            actor.id()
                        ))
                    }
                },
                ["auth"],
            );

            router.put_x("/cel/{name}", |
            Path(name): Path<String>,
            CtxExt(sandbox): CtxExt<CommonExpressionSandbox>,
             Json(payload): Json<CelPayload>,
             | async move {

                sandbox.set_program(&name, &payload.source);
                // if let Some(attributes) = payload.attributes.cloned() {
                //     //
                // }


                 Json(payload) 
            });

            router.post_x("/resource-meta", |
              CtxExt(manager): CtxExt<Manager>,
              Json(resource_meta): Json<ResourceMeta>,
            |async move {
                let mut repo = ResourceMetaRepo::new(&manager);

               if let Ok(saved) = repo.insert(resource_meta).await {
                   Json(Some(saved)) 
               } else {
                  Json(None)
               }
            });
        });
    }
}


#[derive(serde::Deserialize, serde::Serialize)]
struct CelPayload{
    attributes: Option<serde_json::Value>,
    source: String
}

#[derive(Debug, Default, Clone,  DirtyTable, serde::Deserialize, serde::Serialize)]
#[dirty(table = "resources_metadata")]
struct ResourceMeta {
    #[serde(skip_deserializing)]
    id: Option<i64>,
    resource_id: String,
    metadata: JsonField
}