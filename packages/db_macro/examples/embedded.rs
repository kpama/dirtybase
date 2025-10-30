use dirtybase_db::{
    TableModel, connector::sqlite::make_sqlite_in_memory_manager, types::ToColumnAndValue,
};
use dirtybase_db_macro::DirtyTable;

#[tokio::main]
async fn main() {
    let p = Parent {
        name: "Parent Name".to_string(),
        child: Child {
            child_field: "child_field value".to_string(),
        },
        json_child: Child {
            child_field: "embedded child_field value".to_string(),
        },
    };
    let x = p.to_column_value();

    let _manager = make_sqlite_in_memory_manager().await;
    println!("{x:#?}");
    println!("columns: {:#?}", Parent::table_columns());

    // println!(Parent::child_column_name_for_child_field());
}

#[derive(Debug, Default, Clone, DirtyTable)]
#[dirty(id = "child_field", no_timestamp, no_soft_delete)]
struct Child {
    child_field: String,
}

#[derive(Debug, Default, Clone, DirtyTable)]
#[dirty(id = "name", no_timestamp, no_soft_delete)]
struct Parent {
    name: String,
    #[dirty(flatten)]
    child: Child,
    #[dirty(embedded)]
    json_child: Child,
}
