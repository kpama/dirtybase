use dirtybase_contract::prelude::{
    PermStorageProvider, PermissionStorage, PermissionStorageResolver,
    model::{
        Actor, ActorRepo, ActorRole, FetchActorOption, FetchActorPayload, Permission,
        PersistActorPayload, PersistActorRolePayload, PersistPermissionPayload, PersistRolePayload,
        PersistRolePermission, Role, RolePermission,
    },
};
use dirtybase_db::{TableModel, base::manager::Manager, types::ArcUuid7};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        // .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");

    let app_service = dirtybase_app::setup()
        .await
        .expect("could not setup application");

    app_service.init().await;

    let context = app_service.global_context().await;

    let manager: Manager = context.get().await.expect("could not get db manager");

    let mut actor_repo = ActorRepo::new(&manager);

    let permission_storage = context
        .get::<PermStorageProvider>()
        .await
        .expect("could not get permission storage provider");

    let payload = FetchActorPayload::ById {
        id: "019c309d-017d-7742-9e71-5e95f97db10c"
            .try_into()
            .expect("could not create UUID7"),
    };
    let options = FetchActorOption {
        with_roles: true,
        ..Default::default()
    };

    let result = permission_storage.fetch_actor(payload, Some(options)).await;
    // .unwrap();
    tracing::error!("actor result: {:#?}", result);
    // tracing::debug!(
    //     "an actor with his roles: {}",
    //     serde_json::to_string_pretty(&result.unwrap()).unwrap()
    // );

    // let role_perm = PersistRolePermission::Save {
    //     record: RolePermission::new(
    //         "019c2c3f-2cc2-7070-9fc9-8f64c34ca79c".try_into().unwrap(),
    //         Some("019c2c24-4a07-7bc1-83d3-93b2653b3a85".try_into().unwrap()),
    //         None,
    //     ),
    // };
    // _ = permission_storage.save_role_permission(role_perm).await;

    return;
    match actor_repo.first().await {
        Ok(Some(a)) => {
            let payload = FetchActorPayload::ById {
                id: a.id().unwrap().clone(),
            };
            let actor = permission_storage.fetch_actor(payload, None).await;
            println!("fetched an Actor by Id: {:#?}", actor);
        }
        _ => {
            let actor = Actor::default();
            let actor_payload = PersistActorPayload::Save { actor };
            if let Ok(Some(actor)) = permission_storage.save_actor(actor_payload).await {
                tracing::info!("actor Id: {:#?}", actor.id().unwrap());
                // create roles
                let role = Role::new("manager", "Manager");
                if let Ok(Some(role)) = permission_storage
                    .save_role(PersistRolePayload::Save { role })
                    .await
                {
                    let actor_role =
                        ActorRole::new(actor.id().unwrap().clone(), role.id().unwrap().clone());
                    let result = permission_storage
                        .save_actor_role(PersistActorRolePayload::Save { record: actor_role })
                        .await;
                    tracing::info!("actor role: {:#?}", result);
                }
            }
        }
    };
}
