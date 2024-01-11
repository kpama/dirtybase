use std::collections::HashMap;

use dirtybase_db::{
    base::{manager::Manager, schema::DatabaseKind},
    config::BaseConfig,
    field_values::FieldValue,
    types::FromColumnAndValue,
    TableEntityTrait,
};
use dirtybase_db_macro::DirtyTable;

#[derive(DirtyTable, Default, Debug)]
#[dirty(id = "id", table = "people")]
struct Person {
    id: Option<i64>,
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

#[tokio::main]
async fn main() {
    let mut db_result = HashMap::new();
    let config = dirtybase_db::dirtybase_config::DirtyConfig::default();
    let mut db_config = dirtybase_db::config::DirtybaseDbConfig::new(&config).await;

    db_config.sqlite_write = Some(BaseConfig {
        enable: true,
        url: "sqlite://:memory:".to_string(),
        max: 2,
        sticky: None,
        sticky_duration: None,
        foreign_key: Some(true),
        busy_timeout: None,
    });
    db_config.default = Some(DatabaseKind::Sqlite);

    db_result.insert("id".to_owned(), FieldValue::I64(66));
    db_result.insert("is_admin".to_owned(), FieldValue::Boolean(false));

    let pool_manager = dirtybase_db::setup_using(db_config).await;

    let manager = pool_manager.schema_manger(&DatabaseKind::Sqlite).unwrap();
    create_tables(&manager).await;

    // insert
    let person1 = Person {
        id: None,
        first_name: "Frist Name Value".to_string(),
        last_name: "Last Name Value".to_string(),
        is_admin: true,
        orders: Vec::new(),
    };
    let person2 = Person {
        id: None,
        first_name: "Frist Name Value 2".to_string(),
        last_name: "Last Name Value 2".to_string(),
        is_admin: false,
        orders: Vec::new(),
    };
    let person3 = Person {
        id: None,
        first_name: "Frist Name Value 3".to_string(),
        last_name: "Last Name Value 3".to_string(),
        is_admin: false,
        orders: Vec::new(),
    };

    let mut v = Vec::new();
    v.push(person3.into_column_for_first_name().unwrap());
    v.push(person3.into_column_for_last_name().unwrap());
    v.push(person3.into_column_for_is_admin().unwrap());

    // manager
    //     .raw_insert(
    //         "insert into people (first_name, last_name, is_admin) values (?, ?, ?)",
    //         v,
    //     )
    //     .await;

    _ = manager.insert(Person::table_name(), person1).await;
    _ = manager.insert(Person::table_name(), person2).await;

    let mut query = manager.table(Person::table_name());
    // let r1 = query.eq(Person::col_name_for_id(), 2).get().await;

    if let Ok(Some(result2)) = manager
        .select_from_table("people", |q| {
            let v = [1, 4];
            q.and_is_in("id", v.to_vec());
        })
        .get()
        .await
    {
        for row in result2 {
            println!("Id: {:?}", row.get("id").unwrap());
        }
    }

    // query
    let repo = Person::repo(manager);
    let result = repo.all().await;

    dbg!(result);
    // dbg!(result2);
}

async fn create_tables(manager: &Manager) {
    manager
        .create_table_schema("people", |builder| {
            builder.id(Person::id_column());
            builder.string(Person::col_name_for_first_name());
            builder.string(Person::col_name_for_last_name());
            builder.boolean(Person::col_name_for_is_admin());
        })
        .await;
}

/*
struct Person {
    id: i64,
    first_name: String,
    last_name: String,
    is_admin: bool,
    #[dirty(skip)]
    orders: Vec<Order>,
}
*/
