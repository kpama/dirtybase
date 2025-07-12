use std::hash::{DefaultHasher, Hash, Hasher};

use dirtybase_app::setup;
use dirtybase_db::{
    TableModel,
    base::{manager::Manager, query::QueryBuilder},
    field_values::FieldValue,
    types::DateTimeField,
};
use dirtybase_db_macro::DirtyTable;
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(Level::TRACE)
        .try_init()
        .expect("could not setup tracing");

    let app_service = setup().await.unwrap();

    _ = app_service.init().await;

    if let Ok(manager) = app_service.global_context().await.get::<Manager>().await {
        let mut author_repo = AuthorRepo::new(&manager);
        println!(
            "author: {:#?}",
            author_repo.with_posts().by_id(1).get().await
        );

        manager.close().await;
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

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(table = "posts")]
struct Post {
    id: i64,
    title: String,
    author_id: i64,
}

impl Post {
    pub fn author_id_hash(&self) -> u64 {
        calculate_hash(&self.author_id)
    }
}

#[derive(Debug, Default, DirtyTable)]
#[dirty(table = "authors")]
struct Author {
    id: i64,
    first_name: String,
    last_name: String,
    #[dirty(rel(kind = "has_many"))]
    posts: Vec<Post>,
}

struct AuthorRelation {
    builder: QueryBuilder,
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

impl AuthorRepo {
    pub fn by_id(&mut self, id: impl Into<FieldValue>) -> &mut Self {
        self.builder
            .is_eq(Author::prefix_with_tbl(Author::col_name_for_id()), id);
        self
    }
}
