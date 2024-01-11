use dirtybase_db::TableEntityTrait;
use dirtybase_db_macro::DirtyTable;

#[derive(Debug, Default, Clone, DirtyTable)]
struct User {
    name: String,
    points: Vec<u64>,
}

fn main() {
    println!("columns: {:#?}", User::table_columns());
}
