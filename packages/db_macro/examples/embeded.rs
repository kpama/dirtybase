use dirtybase_db::{connector::sqlite::make_sqlite_in_memory_manager, types::IntoColumnAndValue};
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
    let x = p.into_column_value();

    let manager = make_sqlite_in_memory_manager().await;
    dbg!("{:#?}", x);
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Child {
    child_field: String,
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Parent {
    name: String,
    #[dirty(flatten)]
    child: Child,
    #[dirty(embedded)]
    json_child: Child,
}
