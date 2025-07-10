#![allow(dead_code)]

use dirtybase_db_macro::DirtyTable;

#[tokio::main]
async fn main() {}

#[derive(Debug, Default, Clone, DirtyTable)]
#[dirty(table = "family", id = "id")]
struct Family {
    id: String,
    #[dirty(rel(kind = "has_many", foreign_key = "family_id", local_key = "id"))]
    members: Option<Vec<Member>>,
}

#[derive(Debug, Default, Clone, DirtyTable)]
#[dirty(table = "member", id = "id")]
struct Member {
    id: String,
    family_id: String,
    // #[dirty(rel(kind = "belongs_to", foreign_key = "id", local_key = "family_id"))]
    // family: Option<Family>,
    // #[dirty(rel(kind = "has_one"))]
    // person: Option<Person>,
}

#[derive(Debug, Default, Clone, DirtyTable)]
#[dirty(table = "people", id = "id")]
pub(crate) struct Person {
    id: String,
    member_id: String,
}
