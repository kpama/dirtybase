use dirtybase_app::setup;
use dirtybase_db::{base::manager::Manager, query_values::QueryValue, types::DateTimeField};
use dirtybase_db_macro::DirtyTable;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        // .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");

    let app_service = setup().await.unwrap();

    _ = app_service.init().await;

    if let Ok(manager) = app_service.global_context().await.get::<Manager>().await {
        let result = manager
            .select_from_table("posts", |q| {
                q.is_in("posts.id", vec![4, 1]);
                q.select_multiple(&["posts.id", "title", "date"]);
                q.select_as("author_id", "author_id");
                q.subquery_column(
                    "authors",
                    |q1| {
                        q1.select("first_name");
                        q1.is_eq("id", QueryValue::ColumnName("author_id".to_string()));
                    },
                    Some("first_name"),
                );
            })
            .fetch_all_to::<PostWithAuthor>()
            .await;

        println!("result 1: {:#?}", result);

        let result = manager
            .select_from_table("posts", |q| {
                q.is_in("posts.id", vec![4, 1]);
                q.select_multiple(&["posts.id", "title", "date"]);
                q.left_join_and_select(
                    "authors",
                    "posts.author_id",
                    "=",
                    "authors.id",
                    &["authors.first_name"],
                );
            })
            .fetch_all_to::<PostWithAuthor>()
            .await;

        println!("result 2: {:#?}", result);

        manager.close().await;
        println!("db closed");
    }

    println!(">>>> finished <<<<< ");
}

#[derive(Debug, Default, DirtyTable)]
#[dirty(table = "posts")]
struct PostWithAuthor {
    id: i64,
    title: String,
    date: DateTimeField,
    first_name: String,
}
