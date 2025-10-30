#![allow(dead_code)]

use dirtybase_db::types::{CreatedAtField, DeletedAtField, UpdatedAtField};
use dirtybase_db_macro::DirtyTable;

#[tokio::main]
async fn main() {}

#[derive(Debug, Default, Clone, DirtyTable)]
#[dirty(
    id_column = "foo",
    updated_at = "updated_on",
    created_at = "created_on",
    deleted_at = "deleted_on",
    table = "my_posts"
)]
struct Post {
    id: Option<i64>,
    #[dirty(col=post_title)]
    title: String,
    content: String,
    updated_on: UpdatedAtField,
    created_on: CreatedAtField,
    deleted_on: DeletedAtField,
}
