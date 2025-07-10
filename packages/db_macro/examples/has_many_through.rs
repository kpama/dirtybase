use dirtybase_db::{
    TableModel, base::manager::Manager, connector::sqlite::make_sqlite_in_memory_manager,
};
use dirtybase_db_macro::DirtyTable;
use rand::distr::SampleString;

#[tokio::main]
async fn main() {
    let manager = make_sqlite_in_memory_manager().await;
    setup_db(&manager).await;

    let mut customer_repo = CustomerRepo::new(&manager);

    println!("{:#?}", customer_repo.with_invoices().get().await);
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Customer {
    id: Option<i64>,
    name: String,
    #[dirty(rel(kind = has_many_through, pivot: Order, pivot_through_col = id, through_col= order_id))]
    invoices: Option<Vec<Invoice>>,
    #[dirty(rel(kind = has_many))]
    orders: Option<Vec<Order>>,
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Order {
    id: Option<i64>,
    customer_id: i64,
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Invoice {
    id: Option<i64>,
    order_id: i64,
    total: i64,
}

async fn setup_db(manager: &Manager) {
    create_tables(manager).await;
    seed_tables(manager).await;
}

async fn create_tables(manager: &Manager) {
    _ = manager
        .create_table_schema(Customer::table_name(), |table| {
            table.id(None);
            table.string(Customer::col_name_for_name());
        })
        .await;

    _ = manager
        .create_table_schema(Order::table_name(), |table| {
            table.id(None);
            table.id_table_fk::<Customer>(true);
        })
        .await;

    _ = manager
        .create_table_schema(Invoice::table_name(), |table| {
            table.id(None);
            table.integer(Invoice::col_name_for_total());
            table.id_table_fk::<Order>(true);
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

        for _ in 1..=5 {
            _ = manager
                .insert_into::<Order>(Order {
                    customer_id: c,
                    ..Default::default()
                })
                .await;
        }

        if let Ok(Some(orders)) = manager
            .select_from::<Order>(|q| {
                q.is_eq(Order::col_name_for_customer_id(), c);
            })
            .fetch_all_to::<Order>()
            .await
        {
            for an_order in orders {
                _ = manager
                    .insert_into::<Invoice>(Invoice {
                        order_id: an_order.id.clone().unwrap(),
                        total: rand::random_range(50..=10000),
                        ..Default::default()
                    })
                    .await;
            }
        }
    }
}
