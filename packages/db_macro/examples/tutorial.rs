#![allow(dead_code)]

use dirtybase_db_macro::DirtyTable;

#[tokio::main]
async fn main() {}

#[derive(Debug, Default, DirtyTable)]
#[dirty(id_column="foo",updated_at=updated_on, created_at="created_on", deleted_at=deleted_on, table=my_posts)]
struct Post {
    id: Option<i64>,
    #[dirty(col=post_title)]
    title: String,
    content: String,
}
