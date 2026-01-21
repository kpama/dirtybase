use std::sync::Arc;

use dirtybase_auth::helpers::get_auth_storage;
use dirtybase_auth::memory_storage::AuthUserMemoryStorage;
use dirtybase_contract::{
    ExtensionSetup,
    auth_contract::{AuthUser, AuthUserPayload, AuthUserStatus, Gate, GateResponse},
    prelude::{Context, StatusCode},
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        // .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");

    let context = Context::make_global().await;
    dirtybase_auth::Extension::default().setup(&context).await;

    //
    Gate::before(|_actor: AuthUser| async {
        // -
        if false {
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
            println!("post: {post:#?}");
            StatusCode::FORBIDDEN.into()
        },
    )
    .await;
    register_gates().await;

    let storage = get_auth_storage(context.clone(), Some(AuthUserMemoryStorage::NAME))
        .await
        .expect("could not resolve the auth storage provider");

    let mut payload = AuthUserPayload::new();
    payload.status = AuthUserStatus::Active.into();

    if let Ok(user) = storage.store(payload).await {
        println!("current user: {:?}", &user.username());
        context.set(user).await;
        context.set((Arc::new(Post::default()),)).await;

        let gate = Gate::from(&context);
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
    Gate::define(PostPermission::DeleteRecords, || async {
        //
        GateResponse::allow().into()
    })
    .await;
}

#[derive(Debug)]
#[allow(unused)]
struct Post {
    id: i32,
}

impl Default for Post {
    fn default() -> Self {
        Self { id: 44 }
    }
}

enum PostPermission {
    DeleteRecords,
}

impl AsRef<str> for PostPermission {
    fn as_ref(&self) -> &str {
        match self {
            PostPermission::DeleteRecords => "can-delete-records",
        }
    }
}
