use dirtybase_contract::auth_contract::{Actor, ActorRepo, PermissionManager};
use dirtybase_db::{
    TableModel,
    base::{cursor_builder::CursorBuilder, manager::Manager},
};
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

    if let Ok(per_manager) = context.get::<PermissionManager>().await {
        println!(
            "can create resource 0: {}",
            per_manager.can("resource_0:create")
        );
        println!(
            "can create resource 10: {}",
            per_manager.can("resource_10:create")
        );
    }

    return;

    let mut actor_repo = ActorRepo::new(&manager);

    let mut cursor = CursorBuilder::new(Actor::col_name_for_id(), None);
    cursor.set_limit(5);
    let result = actor_repo
        .with_roles()
        .with_actor_roles()
        .cursor_paginate(Some(cursor))
        .await;
    println!("{:#?} => {:#?}", result.cursor_ref(), result.data_ref());
    return;
    let mut cursor = CursorBuilder::new(Actor::col_name_for_id(), None);
    cursor.set_limit(4);

    let mut page = manager
        .select_from::<Actor>(|_q| {})
        .cursor_paginate_to::<Actor>(cursor)
        .await;
    let mut counter = 1;
    loop {
        if let Ok(list) = page.data_ref() {
            if list.is_empty() {
                break;
            }

            for actor in list {
                println!("id: {}", actor.id().as_ref().unwrap(),);
            }
        } else {
            break;
        }
        counter += 1;
        println!("fetch next page: {}", counter);
        page = manager
            .select_from::<Actor>(|_| {})
            .cursor_paginate_to::<Actor>(page.cursor())
            .await;
    }
}
