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

    let result = actor_repo.with_roles().first().await;

    println!("{result:#?}");
}
