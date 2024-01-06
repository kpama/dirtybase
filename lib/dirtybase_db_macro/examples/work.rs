use std::collections::HashMap;

use dirtybase_db_macro::DirtyTable;
use dirtybase_db_types::{field_values::FieldValue, types::FromColumnAndValue};

#[derive(DirtyTable, Default, Debug)]
#[dirty(id = "id", table = "people")]
struct Person {
    id: i64,
    first_name: String,
    last_name: String,
    is_admin: bool,
    #[dirty(skip)]
    orders: Vec<Order>,
}

#[derive(Debug, DirtyTable, Default)]
#[dirty(id = "id", table = "orders")]
struct Order {
    id: i64,
    #[dirty(skip)]
    items: Vec<String>,
}

fn main() {
    let mut db_result = HashMap::new();

    db_result.insert("id".to_owned(), FieldValue::I64(66));
    db_result.insert("is_admin".to_owned(), FieldValue::Boolean(false));

    let person = Person::from_column_value(db_result);

    dbg!(person);
}
