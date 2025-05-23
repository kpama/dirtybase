use std::collections::HashMap;

use axum::{Json, extract::Path};
use dirtybase_contract::http_contract::Bind;

#[tokio::main]
async fn main() {
    let app = dirtybase_app::setup().await.unwrap();

    app.setup_web(|mut route, _| {
        /*
          middleware.bind("post", )
        */
        route.general(None, |router| {
            router.get_x("/posts/{post}", get_post);
        });

        route
    })
    .await;

    Bind::<Post>::resolver(|mut res| async move {
        //
        println!(">>> resolver got called <<< ");
        let mut repo = HashMap::new();
        for id in 1..=100 {
            repo.insert(
                id,
                Post {
                    id,
                    name: format!("post {}", id),
                },
            );
        }

        if let Ok(Path(id)) = res.get_path::<i32>().await {
            if let Some(post) = repo.get(&id).cloned() {
                return Some(post.into());
            }
        }
        None
    })
    .await;

    _ = dirtybase_app::run(app).await;
}

async fn get_post(Path(_post_id): Path<i32>, Bind(post): Bind<Post>) -> Json<Post> {
    println!("{:#?}", &post);
    Json(post)
}

#[derive(Debug, Clone, serde::Serialize)]
struct Post {
    id: i32,
    name: String,
}
