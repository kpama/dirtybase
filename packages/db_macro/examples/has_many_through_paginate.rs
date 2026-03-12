use dirtybase_db::{
    TableModel, base::manager::Manager, connector::sqlite::make_sqlite_in_memory_manager,
    types::TimestampField,
};
use dirtybase_db_macro::DirtyTable;
use dirtybase_helper::time::current_datetime;
use rand::distr::SampleString;

#[tokio::main]
async fn main() {
    let manager = make_sqlite_in_memory_manager().await;
    setup_db(&manager).await;

    let mut customer_repo = CustomerRepo::new(&manager);

    if let Ok(Some(customer)) = customer_repo.latest().await {
        let mut paginator = customer_repo.invoices_paginate_cursor(&customer);
        println!("page one: {:#?}", paginator.next().await.data_ref());
        println!("page two: {:#?}", paginator.next().await.data_ref());
    }
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Customer {
    id: Option<i64>,
    name: String,
    #[dirty(rel(kind = has_many))]
    orders: Option<Vec<Order>>,
    #[dirty(rel(kind = has_many_through, soft_deletable,  pivot = Order, pivot_through_col = id, through_col= order_id))]
    invoices: Option<Vec<Invoice>>,
}

#[derive(Debug, Default, Clone, DirtyTable)]
#[dirty(soft_deletable)]
struct Order {
    id: Option<i64>,
    customer_id: i64,
    deleted_at: Option<TimestampField>,
}

#[derive(Debug, Default, Clone, DirtyTable)]
#[dirty(soft_deletable)]
struct Invoice {
    id: Option<i64>,
    order_id: i64,
    total: i64,
    deleted_at: Option<TimestampField>,
}

async fn setup_db(manager: &Manager) {
    create_tables(manager).await;
    seed_tables(manager).await;
}

async fn create_tables(manager: &Manager) {
    _ = manager.drop_table(Invoice::table_name()).await;
    _ = manager.drop_table(Order::table_name()).await;
    _ = manager.drop_table(Customer::table_name()).await;
    _ = manager
        .create_table_schema(Customer::table_name(), |table| {
            table.id(None);
            table.string(Customer::col_name_for_name());
            table.soft_deletable();
        })
        .await;

    _ = manager
        .create_table_schema(Order::table_name(), |table| {
            table.id(None);
            table.id_table_fk::<Customer>(true);
            table.soft_deletable();
        })
        .await;

    _ = manager
        .create_table_schema(Invoice::table_name(), |table| {
            table.id(None);
            table.integer(Invoice::col_name_for_total());
            table.id_table_fk::<Order>(true);
            table.soft_deletable();
        })
        .await;
}

async fn seed_tables(manager: &Manager) {
    for c in 1..=5 {
        _ = manager
            .insert_into::<Customer>(Customer {
                name: rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 10),
                ..Default::default()
            })
            .await;

        for _ in 1..=rand::random_range(5..=10) {
            if let Err(e) = manager
                .insert_into::<Order>(Order {
                    customer_id: c,
                    ..Default::default()
                })
                .await
            {
                println!("error creating order: {}", e);
            }
        }

        if let Ok(orders) = manager
            .select_from::<Order>(|q| {
                q.is_eq(Order::col_name_for_customer_id(), c);
            })
            .fetch_all_to::<Order>()
            .await
        {
            for an_order in orders {
                _ = manager
                    .insert_into::<Invoice>(Invoice {
                        order_id: an_order.id.unwrap(),
                        total: rand::random_range(50..=10000),
                        deleted_at: if rand::random_bool(1.0 / 3.0) {
                            Some(current_datetime())
                        } else {
                            None
                        },
                        ..Default::default()
                    })
                    .await;
            }
        }
    }
}
