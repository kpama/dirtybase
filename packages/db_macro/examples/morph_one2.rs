use dirtybase_db::{
    TableModel, base::manager::Manager, connector::sqlite::make_sqlite_in_memory_manager,
};
use dirtybase_db_macro::DirtyTable;
use rand::distr::SampleString;

#[tokio::main]
async fn main() {
    let manager = make_sqlite_in_memory_manager().await;
    setup_db(&manager).await;
    let mut post_repo = PostRepo::new(&manager);
    println!("{:#?}", post_repo.with_image().get().await);
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Post {
    id: Option<i64>,
    title: String,
    #[dirty(rel(kind = "morph_one", morph_name = "imageable", morph_type = "post",))]
    image: Option<Image>,
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Image {
    id: Option<i64>,
    location: String,
    imageable_id: i64,
    imageable_type: String,
}

async fn setup_db(manager: &Manager) {
    create_tables(manager).await;
    seed_tables(manager).await;
}

async fn create_tables(manager: &Manager) {
    _ = manager
        .create_table_schema(Post::table_name(), |table| {
            table.id(None);
            table.string(Post::col_name_for_title());
            table.soft_deletable();
        })
        .await;

    _ = manager
        .create_table_schema(Image::table_name(), |table| {
            table.id(None);
            table.string(Image::col_name_for_location());
            table.integer(Image::col_name_for_imageable_id());
            table.string(Image::col_name_for_imageable_type());
            table.soft_deletable();
            table.index(&[
                Image::col_name_for_imageable_id(),
                Image::col_name_for_imageable_type(),
            ]);
        })
        .await;
}

async fn seed_tables(manager: &Manager) {
    for _ in 1..=5 {
        let title = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 10);
        _ = manager
            .insert(
                Post::table_name(),
                Post {
                    title: title.clone(),
                    ..Default::default()
                },
            )
            .await;

        if let Ok(Some(post)) = manager
            .select_from::<Post>(|q| {
                q.is_eq(Post::col_name_for_title(), &title);
            })
            .fetch_one_to::<Post>()
            .await
        {
            _ = manager
                .insert(
                    Image::table_name(),
                    Image {
                        imageable_type: PostRepo::imageable_type().to_string(),
                        imageable_id: post.id.unwrap(),
                        location: rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 10),
                        ..Default::default()
                    },
                )
                .await;
        }
    }
}
