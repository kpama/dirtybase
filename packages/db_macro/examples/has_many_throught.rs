use dirtybase_db::{
    TableEntityTrait, base::manager::Manager, connector::sqlite::make_sqlite_in_memory_manager,
};
use dirtybase_db_macro::DirtyTable;
use rand::distr::SampleString;

#[tokio::main]
async fn main() {}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Customer {
    id: Option<i64>,
    name: String,
    #[dirty(rel(kind = has_many))]
    orders: Option<Vec<Order>>,
    #[dirty(rel(kind = has_many_through, pivot: Order))]
    invoices: Option<Vec<Invoice>>,
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
    total: u64,
}
