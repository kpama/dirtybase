use dirtybase_db::{TableEntityTrait, connector::sqlite::make_sqlite_in_memory_manager};
use dirtybase_db_macro::DirtyTable;

#[derive(Debug, Default, Clone, DirtyTable)]
struct User {
    name: String,
    // name2: Option<String>,
    // n1: i64,
    // points: Vec<u64>,
}

#[tokio::main]
async fn main() {
    let manager = make_sqlite_in_memory_manager().await;

    manager
        .create_table_schema(User::table_name(), |table| {
            table.string(User::col_name_for_name());
        })
        .await;

    for name in ["a", "b", "c", "d"] {
        manager
            .insert(
                User::table_name(),
                User {
                    name: name.to_string(),
                },
            )
            .await;
    }

    println!("columns: {:#?}", User::table_columns());
}
