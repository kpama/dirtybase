use dirtybase_auth::Gate;

#[tokio::main]
async fn main() {
    let mut g = Gate::new().await;
    let user = User {
        id: 100,
        name: "user_one".to_string(),
        role: "admin".to_string(),
    };

    Gate::define("can_read_x", |user: User, id: i32| async move {
        //-
        id == 25 && user.role == "admin"
    })
    .await;
    Gate::define("can_edit_post", |user: User, post: Post| async move {
        //-
        post.owner == user.id
    })
    .await;

    g.set(25).await;
    g.set(user).await;
    g.set(Post { id: 6, owner: 100 }).await;

    println!("g1 allow: {}", g.allows("can_read_x").await);
    println!("can edit post : {}", g.allows("can_edit_post").await);
    println!("g1 allow when: {}", g.allows_when("a", (66_f32,)).await);

    // let mut g2 = Gate::new().await;
    // println!("g2: {}", g2.allows("a").await);
}

#[derive(Debug, Clone)]
struct User {
    id: i32,
    name: String,
    role: String,
}

#[derive(Debug, Clone)]
struct Post {
    id: i32,
    owner: i32,
}
