use dirtybase_db_macro::DirtyTable;
use dirtybase_db_types::TableEntityTrait;

#[derive(Debug, Default, Clone, DirtyTable)]
struct User {
    name: String,
    points: Vec<u64>,
}

fn main() {
    println!("columns: {:#?}", User::table_columns());
}
