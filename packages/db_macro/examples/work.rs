#![allow(dead_code)]

use dirtybase_db::{
    TableModel, base::manager::Manager, connector::sqlite::make_sqlite_in_memory_manager,
};
use dirtybase_db_macro::DirtyTable;

#[derive(DirtyTable, Default, Debug)]
#[dirty(id = "id", table = "users")]
struct Person {
    id: Option<i64>,
    is_admin: bool,
    first_name: String,
    last_name: String,
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
    setup_db().await;
    // let people_repo = Person::repo_instance().await;
    // insert
    let _person1 = Person {
        id: None,
        first_name: "Person1 Frist".to_string(),
        last_name: "Person2 Last".to_string(),
        is_admin: true,
        orders: Vec::new(),
    };

    let _person2 = Person {
        id: None,
        first_name: "Person2 Frist".to_string(),
        last_name: "Person2 Last".to_string(),
        is_admin: true,
        orders: Vec::new(),
    };

    // _ = people_repo.insert(person1).await;
    // _ = people_repo.insert(person2).await;

    // let results = people_repo.ids(vec![4, 1, 5, 2]).await;
    // dbg!(results);

    // let mut builder = people_repo.builder();
    // builder
    //     .query()
    //     .is_in("id", vec![4, 1, 6, 7])
    //     .and_eq("id", 1);

    // if let Ok(Some(mut entity)) = builder.one().await {
    //     entity.first_name = "I got changed!!!".to_string();
    //     let result = people_repo.update(entity, 1, None).await;
    //     println!("did a query via the builder: {:#?}", result);
    // }
}

async fn setup_db() {
    let manager = make_sqlite_in_memory_manager().await;
    create_tables(&manager).await;
}

async fn create_tables(manager: &Manager) {
    _ = manager
        .create_table_schema(Person::table_name(), |builder| {
            builder.id(Person::id_column().into());
            builder.string(Person::col_name_for_first_name());
            builder.string(Person::col_name_for_last_name());
            builder.boolean(Person::col_name_for_is_admin());
        })
        .await;
}
