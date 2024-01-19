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

    _ = manager.insert(Person::table_name(), person1).await;

    let results = Person::repo(manager).first_name_like("%Value").await;

    dbg!(results);
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
