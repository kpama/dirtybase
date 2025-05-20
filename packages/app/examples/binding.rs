use std::collections::HashMap;

use axum::{
    RequestExt,
    extract::{FromRequest, FromRequestParts, Path},
    response::IntoResponse,
};
use dirtybase_contract::{http_contract::Bind, prelude::CtxExt};

#[tokio::main]
async fn main() {
    let app = dirtybase_app::setup().await.unwrap();

    app.setup_web(|mut route, _| {
        /*
          middleware.bind("post", )
        */
        route.general(None, |router| {
            router.get_x_with_middleware(
                "/posts/{post}",
                get_post,
                [
                    "bind::posts>path=post,field=id",
                    // "bind::user_finder>path=user,field=id",
                ], // use the bind middleware, call the "post_finder", pass everything after the equal to it
            );
        });

        route
    })
    .await;

    Bind::<Post>::resolver(|res| async {
        //
        println!(">>> resolver got called <<< ");
        Some(
            Post {
                id: 43,
                name: "works...".to_string(),
            }
            .into(),
        )
    })
    .await;

    _ = dirtybase_app::run(app).await;
}

async fn get_post(Path(post_id): Path<i32>, Bind(post): Bind<Post>) -> impl IntoResponse {
    println!("{:#?}", post);
    format!("post id: {}", post_id)
}

#[derive(Debug, Clone)]
struct Post {
    id: i32,
    name: String,
}
