use std::sync::Arc;

use dirtybase_auth::{MEMORY_STORAGE, helpers::get_auth_storage};
use dirtybase_contract::{
    ExtensionSetup,
    auth_contract::{AuthUser, AuthUserPayload, AuthUserStatus, Gate, GateResponse},
    prelude::{Context, StatusCode},
};

#[tokio::main]
async fn main() {
    let context = Context::make_global().await;
    dirtybase_auth::Extension::default().setup(&context).await;

    Gate::before(|_actor: AuthUser| async {
        // -
        if false {
            println!("very wrong....");
            return true.into();
        }
        None
    })
    .await;

    Gate::define(
        "update-post",
        |user: AuthUser, (post,): (Arc<Post>,)| async move {
            //
            println!("is status: {}", user.status());
            println!("post: {:#?}", post);
            StatusCode::FORBIDDEN.into()
        },
    )
    .await;
    register_gates().await;

    let storage = get_auth_storage(context.clone(), Some(MEMORY_STORAGE))
        .await
        .expect("could not resolve the auth storage provider");

    let mut payload = AuthUserPayload::new();
    payload.email = "foo@bar.com".to_string().into();
    payload.password = "password".to_string().into();
    payload.username = "testuser".to_string().into();
    payload.status = AuthUserStatus::Active.into();

    if let Ok(user) = storage.store(payload).await {
        println!("current user: {:?}", &user);
        context.set(user).await;
        context.set((Arc::new(Post::default()),)).await;

        let gate = Gate::new();
        if gate
            .all_when(&["update-post"], (Arc::new(Post::default()), 55))
            .await
        {
            println!("you are allow to update post")
        }

        println!(
            "update-posts: {}",
            gate.all_when(&["update-post"], (Arc::new(Post::default()), 55))
                .await
        );
        println!(
            "can-delete-record: {}",
            gate.allows("can-delete-records").await
        );
    }
}

async fn register_gates() {
    Gate::define("can-delete-records", || async {
        //
        GateResponse::allow().into()
    })
    .await;
}

#[derive(Debug)]
struct Post {
    id: i32,
}

impl Default for Post {
    fn default() -> Self {
        Self { id: 44 }
    }
}
