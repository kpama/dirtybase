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

    // let mut data = ColumnAndValue::new();
    // data.insert(
    //     Invoice::col_name_for_deleted_at().to_string(),
    //     dirtybase_helper::time::current_datetime().into(),
    // );
    // _ = manager
    //     .update_table::<Invoice>(data, |q| {
    //         q.is_in(Invoice::col_name_for_id(), vec![1, 3]);
    //     })
    //     .await;

    println!(
        "{:#?}",
        customer_repo.with_orders().with_invoices().get().await
    );
}

#[derive(Debug, Default, Clone, DirtyTable)]
#[dirty(no_timestamp, no_soft_delete)]
struct Customer {
    id: Option<i64>,
    name: String,
    #[dirty(rel(kind = has_many))]
    orders: Option<Vec<Order>>,
    #[dirty(rel(kind = has_many_through,  pivot = Order, pivot_through_col = id, through_col= order_id))]
    invoices: Option<Vec<Invoice>>,
}

#[derive(Debug, Default, Clone, DirtyTable)]
#[dirty(no_timestamp, no_soft_delete)]
struct Order {
    id: Option<i64>,
    customer_id: i64,
}

#[derive(Debug, Default, Clone, DirtyTable)]
#[dirty(no_timestamp)]
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
