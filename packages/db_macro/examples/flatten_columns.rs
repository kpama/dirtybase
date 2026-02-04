use dirtybase_db::TableModel;
use dirtybase_db_macro::DirtyTable;

#[tokio::main]
async fn main() {
    println!("columns: {:#?}", Parent::table_columns());
}

#[derive(Debug, Default, Clone, DirtyTable)]
#[dirty(id = "child_field_a", no_timestamp, no_soft_delete)]
struct Child {
    child_field_a: String,
    child_field_b: String,
    child_field_c: String,
}

#[derive(Debug, Default, Clone, DirtyTable)]
#[dirty(no_timestamp, no_soft_delete)]
struct Parent {
    id: i64,
    #[dirty(flatten)]
    child: Child,
}
